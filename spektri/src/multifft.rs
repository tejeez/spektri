// Calculation of multiple FFTs (but not yet in parallel)

use rustfft::num_complex::Complex;

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
pub fn process<'a>(
    fft: &std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    window: &[f32],
    inputs:  impl Iterator<Item = &'a    [Complex<f32>]>,
    //outputs: impl Iterator<Item = &'a mut[Complex<f32>]>,
    outputs: &mut[&mut [Complex<f32>]]
) {
    /*for (input, output) in inputs.zip(outputs.iter_mut()) {
        // RustFFT does transform in-place, so temporarily
        // write the windowed signal into the output buffer
        apply_window(window, input, *output);
        fft.process(*output);
    }*/
    inputs.zip(outputs.iter_mut()).for_each(
    |(input, output)| {
        // RustFFT does transform in-place, so temporarily
        // write the windowed signal into the output buffer
        apply_window(window, input, *output);
        fft.process(*output);
    });
}
