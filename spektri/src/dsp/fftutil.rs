use rustfft::num_complex::Complex;

/* Return a bin from an FFT result buffer with "modulo" indexing.
 *
 * For real-input FFTs, only half of the result is stored
 * since the result is conjugate symmetric.
 * Use the symmetry to return correct values for the other half as well.
 */
pub fn get_bin(fft_result: &[Complex<f32>], fft_size: usize, is_complex: bool, i: isize) -> Complex<f32>
{
    // Modulo
    let m = (((i % fft_size as isize) + fft_size as isize) % fft_size as isize) as usize;

    if is_complex {
        fft_result[m]
    } else {
        if m <= fft_size / 2 {
            fft_result[m]
        } else {
            fft_result[fft_size - m].conj()
        }
    }
}
