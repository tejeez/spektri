// Calculation of multiple FFTs in parallel

use rustfft::{FftPlanner, num_complex::Complex};
use rayon::prelude::*;

pub struct MultiFft {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
}

// Apply a real-valued window function to a complex signal
fn apply_window(
    window: &[f32],
    input: &[Complex<f32>],
    output: &mut[Complex<f32>]
) {
    for (o, (i, w)) in output.iter_mut().zip(input.iter().zip(window.iter())) {
        *o = Complex{
            re: i.re * w,
            im: i.im * w
        };
    }
}

impl MultiFft {
    pub fn init(fft_size: usize) -> MultiFft {
        let mut planner = FftPlanner::new();
        MultiFft {
            fft: planner.plan_fft_forward(fft_size),
        }
    }

    // Calculate multiple FFTs with a window function
    // for complex input signal
    pub fn process_complex(
        &self,
        window: &[f32],
        inputs: &[&[Complex<f32>]],
        outputs: &mut[&mut [Complex<f32>]]
    ) {
        outputs.par_iter_mut().zip(inputs.par_iter()).for_each(
        |(output, input)| {
            // RustFFT does transform in-place, so temporarily
            // write the windowed signal into the output buffer
            apply_window(window, input, *output);
            self.fft.process(*output);
        });
    }

    // Calculate multiple FFTs with a window function
    // for real input signal
    pub fn process_real(
        &self,
        window: &[f32],
        inputs: &[&[f32]],
        outputs: &mut[&mut [Complex<f32>]]
    ) {
        outputs.par_chunks_mut(2).zip(inputs.par_chunks(2)).for_each(
        |(output, input)| {
            // Calculate two real-input-valued FFTs in one complex FFT
            // TODO
        });
    }
}
