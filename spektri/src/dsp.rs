use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;
use std::io::Write;
use rayon::prelude::*;
use crate::multifft;

// Parameters for signal processing
pub struct DspParams {
    //pub complex: bool, // Type of input signal: true for I/Q, false for real (TODO)
    pub fft_size: usize,
    pub scaling: f32, // Scaling of input values
    pub ffts_per_buf: usize,
    //pub fft_overlap: usize, // Fixed for now
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
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
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
        let mut planner = FftPlanner::new();

        let fft_overlap = params.fft_size / 4; // 25% overlap
        let fft_interval = params.fft_size - fft_overlap; // FFT is taken every fft_interval samples

        (DspState {
            fft_size: params.fft_size,
            ffts_per_buf: params.ffts_per_buf,
            fft_interval: fft_interval,
            fft: planner.plan_fft_forward(params.fft_size),
            acc: vec![0.0; params.fft_size],
            accn: 0,
            window: hann_window(params.fft_size, params.scaling),
            fft_result_buf: vec![Complex{re:0.0, im:0.0}; params.fft_size * params.ffts_per_buf],
        }, InputBufferSize {
            overlap: fft_overlap,
            new: fft_interval * params.ffts_per_buf,
            total: fft_overlap + fft_interval * params.ffts_per_buf
        })
    }

    pub fn process(&mut self, input_buffer: &[Complex<f32>]) -> std::io::Result<()> {
        let fft_interval = self.fft_interval;
        let fft_size = self.fft_size;

        // Buffers for FFT results
        let mut resultbufs: Vec<&mut [Complex<f32>]> = self.fft_result_buf.chunks_mut(fft_size).collect();

        multifft::process(
            &self.fft,
            &self.window,
            &(0..self.ffts_per_buf).map(|i|
                &input_buffer[i*fft_interval .. i*fft_interval+fft_size]
            ).collect::<Vec<&[Complex<f32>]>>(),
            &mut resultbufs
        );

        for fft_result in resultbufs.iter() {
            self.acc.par_iter_mut().zip(fft_result.par_iter()).for_each(
                |(acc_bin, fft_bin)|
            {
                *acc_bin += fft_bin.re * fft_bin.re + fft_bin.im * fft_bin.im;
            });
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

