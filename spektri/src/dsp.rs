use rustfft::num_complex::Complex;
use std::f32::consts::PI;
use std::io::Write;
//use rayon::prelude::*;
use crate::multifft::MultiFft;

#[derive(Copy, Clone)]
pub enum SpectrumFormat { U8, U16 }

// Parameters for signal processing
pub struct DspParams {
    pub complex: bool, // Type of input signal: true for I/Q, false for real (TODO)
    pub fft_size: usize,
    pub scaling: f32, // Scaling of input values
    pub ffts_per_buf: usize,
    //pub fft_overlap: usize, // Fixed for now
    pub spectrum_format: SpectrumFormat, // Output format for spectrum data
    pub spectrum_averages: u32, // Number of FFTs averaged
}

// requirements for the buffers given to DspState::process
pub struct InputBufferSize {
    pub new:     usize, // Number of new samples in each buffer
    pub overlap: usize, // Overlampping samples from previous buffer
    pub total:   usize, // Total size of the buffer (new+overlap)
}

pub struct DspState {
    fft_size: usize,
    ffts_per_buf: usize,
    fft_interval: usize,

    mfft: MultiFft,
    accu: SpectrumAccumulator,

    window: Vec<f32>, // Window function,
    fft_result_buf: Vec<Complex<f32>>, // Pre-allocated buffer
}

fn hann_window(size: usize, scaling: f32) -> Vec<f32> {
    let mut w: Vec<f32> = (0..size).map(|i| {
        1.0 - ((i as f32 + 0.5) * 2.0 * PI / size as f32).cos()
    }).collect();

    // Include all the scaling factors into the window function.
    // Normalize sum over the whole window to the desired scaling.
    let s: f32 = scaling / w.iter().sum::<f32>();
    w.iter_mut().for_each(|v| { *v *= s; });

    w
}

impl DspState {
    pub fn init(params: DspParams) -> (DspState, InputBufferSize) {
        let fft_overlap = params.fft_size / 4; // 25% overlap
        let fft_interval = params.fft_size - fft_overlap; // FFT is taken every fft_interval samples
        let result_bins = if params.complex { params.fft_size } else { params.fft_size / 2 + 1 };

        (DspState {
            fft_size: params.fft_size,
            ffts_per_buf: params.ffts_per_buf,
            fft_interval: fft_interval,

            mfft: MultiFft::init(params.fft_size),
            accu: SpectrumAccumulator::init(result_bins, params.spectrum_averages, params.spectrum_format),

            window: hann_window(params.fft_size, params.scaling),
            fft_result_buf: vec![Complex{re:0.0, im:0.0}; result_bins * params.ffts_per_buf],
        }, InputBufferSize {
            overlap: fft_overlap,
            new: fft_interval * params.ffts_per_buf,
            total: fft_overlap + fft_interval * params.ffts_per_buf
        })
    }

    pub fn process_complex(&mut self, input_buffer: &[Complex<f32>]) -> std::io::Result<()> {
        let fft_interval = self.fft_interval;
        let fft_size = self.fft_size;

        // Buffers for FFT results
        let mut resultbufs: Vec<&mut [Complex<f32>]> = self.fft_result_buf.chunks_mut(fft_size).collect();

        self.mfft.process_complex(
            &self.window,
            &(0..self.ffts_per_buf).map(|i|
                &input_buffer[i*fft_interval .. i*fft_interval+fft_size]
            ).collect::<Vec<&[Complex<f32>]>>(),
            &mut resultbufs
        );
        self.accu.accumulate(&resultbufs)?;
        Ok(())
    }

    pub fn process_real(&mut self, input_buffer: &[f32]) -> std::io::Result<()> {
        let fft_interval = self.fft_interval;
        let fft_size = self.fft_size;

        // Buffers for FFT results
        let mut resultbufs: Vec<&mut [Complex<f32>]> = self.fft_result_buf.chunks_mut(fft_size/2+1).collect();

        self.mfft.process_real(
            &self.window,
            &(0..self.ffts_per_buf).map(|i|
                &input_buffer[i*fft_interval .. i*fft_interval+fft_size]
            ).collect::<Vec<&[f32]>>(),
            &mut resultbufs
        );
        self.accu.accumulate(&resultbufs)?;
        Ok(())
    }
}



struct SpectrumAccumulator {
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
    averages: u32, // parameter
    outfmt: SpectrumFormat, // parameter
}

impl SpectrumAccumulator {
    fn init(
        n: usize, // Number of bins
        averages: u32, // Number of FFTs averaged
        outfmt: SpectrumFormat, // Output format for spectrum data
    ) -> SpectrumAccumulator {
        SpectrumAccumulator {
            acc: vec![0.0; n],
            accn: 0,
            averages: averages,
            outfmt: outfmt,
        }
    }

    fn accumulate(
        &mut self,
        fft_results: &[&mut[Complex<f32>]],
        ) -> std::io::Result<()>
    {
        for fft_result in fft_results.iter() {
            // rayon seems to slow it down here. Maybe parallelization could be made optional.
            //self.acc.par_iter_mut().zip(fft_result.par_iter()).for_each(
            self.acc.iter_mut().zip(fft_result.iter()).for_each(
                |(acc_bin, fft_bin)|
            {
                *acc_bin += fft_bin.re * fft_bin.re + fft_bin.im * fft_bin.im;
            });
            self.accn += 1;
            let averages = self.averages;
            if self.accn >= averages {
                // Write the result in binary format into stdout

                let outfmt = self.outfmt;
                let mut printbuf: Vec<u8> = vec![
                    0;
                    self.acc.len() * match outfmt { SpectrumFormat::U16=>2, SpectrumFormat::U8=>1 }];

                // divide accumulator bins by self.accn,
                // but do it as an addition after conversion to dB scale
                let db_plus = (self.accn as f32).log10() * -10.0;

                match outfmt {
                SpectrumFormat::U16 => {
                    for (acc_bin, out) in self.acc.iter().zip(printbuf.chunks_mut(2)) {
                        let db = acc_bin.log10() * 10.0 + db_plus;
                        // quantize to 0.05 dB per LSB, full scale at 4000, clamp to 12 bits
                        let o = (db * 20.0 + 4000.0).max(0.0).min(4095.0) as u16;
                        out[0] = (o >> 8) as u8;
                        out[1] = (o & 0xFF) as u8;
                    }
                },
                SpectrumFormat::U8 => {
                    for (acc_bin, out) in self.acc.iter().zip(printbuf.iter_mut()) {
                        let db = acc_bin.log10() * 10.0 + db_plus;
                        // quantize to 0.5 dB per LSB, full scale at 250
                        *out = (db * 2.0 + 250.0).max(0.0).min(255.0) as u8;
                    }
                }}
                std::io::stdout().write_all(&printbuf)?;

                // Reset accumulator
                for acc_bin in self.acc.iter_mut() {
                    *acc_bin = 0.0;
                }
                self.accn = 0;
            }
        }
        Ok(())
    }
}
