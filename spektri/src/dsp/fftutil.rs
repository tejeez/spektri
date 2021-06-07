use rustfft::num_complex::Complex;

/* Return a bin from an FFT result buffer with "modulo" indexing.
 *
 * For real-input FFTs, only half of the result is stored
 * since the result is conjugate symmetric.
 * Use the symmetry to return correct values for the other half as well.
 */
pub fn get_bin(fft_result: &[Complex<f32>], fft_size: usize, i: isize) -> Complex<f32>
{
    // Modulo
    let m = (((i % fft_size as isize) + fft_size as isize) % fft_size as isize) as usize;

    if m < fft_result.len() {
        // Always true for a complex FFT, since fft_result.len() == fft_size.
        // True for non-negative frequencies of a real-to-complex FFT.
        fft_result[m]
    } else {
        // True for a real-to-complex FFT on negative frequencies.
        // Use conjugate symmetry to return the part which is not stored.
        fft_result[fft_size - m].conj()
    }
}
