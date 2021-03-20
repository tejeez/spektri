// Calculation of multiple FFTs in parallel

use rustfft::num_complex::Complex;
use rayon::prelude::*;

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

// Calculate multiple FFTs with a window function
pub fn process(
    fft: &std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    window: &[f32],
    inputs: &[&[Complex<f32>]],
    outputs: &mut[&mut [Complex<f32>]]
) {
    outputs.par_iter_mut().enumerate().for_each(
    |(i, output)| {
        // RustFFT does transform in-place, so temporarily
        // write the windowed signal into the output buffer
        apply_window(window, inputs[i], *output);
        fft.process(*output);
    });
}
