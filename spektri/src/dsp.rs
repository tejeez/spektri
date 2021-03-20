use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;
use std::io::Write;
use crate::multifft;

// requirements for the buffers given to DspState::process
pub struct InputBufferSize {
    pub new:     usize, // Number of new samples in each buffer
    pub overlap: usize, // Overlampping samples from previous buffer
    pub total:   usize, // Total size of the buffer (new+overlap)
}

pub struct DspState {
    fftsize: usize,
    ffts_per_buf: usize,
    fft_interval: usize,
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
    window: Vec<f32>, // Window function
}

impl DspState {
    pub fn init(fftsize: usize, scaling: f32) -> (DspState, InputBufferSize) {
        let mut planner = FftPlanner::new();

        let ffts_per_buf = 4;
        let fft_overlap = fftsize / 4; // 25% overlap
        let fft_interval = fftsize - fft_overlap; // FFT is taken every fft_interval samples

        (DspState {
            fftsize: fftsize,
            ffts_per_buf: ffts_per_buf,
            fft_interval: fft_interval,
            fft: planner.plan_fft_forward(fftsize),
            acc: vec![0.0; fftsize],
            accn: 0,
            window: {
                let mut w = vec![0.0; fftsize];
                let mut s: f32 = 0.0;
                // Hann window
                for i in 0..fftsize {
                    w[i] = 1.0 - ((i as f32 + 0.5) * 2.0 * PI / fftsize as f32).cos();
                    s += w[i]; // calculate the sum for normalization
                }
                // Include all the scaling factors into the window function.
                // Normalize sum over the whole window to the desired scaling.
                s = scaling / s;
                for i in 0..fftsize {
                    w[i] *= s;
                }
                w
            },
        }, InputBufferSize {
            overlap: fft_overlap,
            new: fft_interval * ffts_per_buf,
            total: fft_overlap + fft_interval * ffts_per_buf
        })
    }

    pub fn process(&mut self, input_buffer: &[Complex<f32>]) -> std::io::Result<()> {
        // Buffers for FFT results
        let mut resultbufbuf = vec![Complex{re:0.0, im:0.0}; self.fftsize * self.ffts_per_buf];
        let mut resultbufs: Vec<&mut [Complex<f32>]> = resultbufbuf.chunks_mut(self.fftsize).collect();

        multifft::process(
            &self.fft,
            &self.window,
            &(0..self.ffts_per_buf).map(|i|
                &input_buffer[i*self.fft_interval .. i*self.fft_interval+self.fftsize]
            ).collect::<Vec<&[Complex<f32>]>>(),
            &mut resultbufs
        );

        for fft_result in resultbufs.iter() {
            for (fft_bin, acc_bin) in fft_result.iter().zip(self.acc.iter_mut()) {
                *acc_bin += fft_bin.re * fft_bin.re + fft_bin.im * fft_bin.im;
            }
            self.accn += 1;
            let averages = 100;
            if self.accn >= averages {
                // Write the result in binary format into stdout

                let mut printbuf: Vec<u8> = vec![0; self.acc.len()];

                // divide accumulator bins by self.accn,
                // but do it as an addition after conversion to dB scale
                let db_plus = (self.accn as f32).log10() * -10.0;

                for (acc_bin, out) in self.acc.iter().zip(printbuf.iter_mut()) {
                    let db = acc_bin.log10() * 10.0 + db_plus;
                    // quantize to 0.5 dB per LSB, full scale at 250
                    *out = (db * 2.0 + 250.0).max(0.0).min(255.0) as u8;
                }
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

