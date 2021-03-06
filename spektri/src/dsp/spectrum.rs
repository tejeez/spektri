/*
 * Spectrum analysis
 *
 * The code here takes FFT results and turns them into
 * spectrum analysis results using the Welch's method.
 *
 * The squared magnitude of each FFT bin is averaged over a number of FFTs.
 * Result of that is then quantized on a logarithmic scale and written
 * into the output file.
 *
 * As a modification from usual Welch's method,
 * the multiplication with a window function before the FFT is replaced
 * by an equivalent circular convolution in frequency domain after the FFT.
 *
 * Normally, this would not be particularly useful, since a convolution
 * requires more multiplications than an equivalent multiplication
 * in the other domain.
 *
 * Here, however, it lets us use a rectangular FFT window, which works
 * better for a fast-convolution filter-bank, allowing the use of the same
 * FFT results for both a filter bank and spectrum analysis.
 */

use rustfft::num_complex::Complex;
use std::io::Write;
use super::fftutil::*;

arg_enum! { // needed for command line parsing
    #[derive(Debug, Copy, Clone)]
    pub enum SpectrumFormat { U8, U16 }
}

pub struct SpectrumAccumulator {
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
    fft_size: usize, // parameter
    averages: u32, // parameter
    outfmt: SpectrumFormat, // parameter
}

impl SpectrumAccumulator {
    pub fn init(
        fft_size: usize,
        complex: bool, // is the input to FFT complex
        averages: u32, // Number of FFTs averaged
        outfmt: SpectrumFormat, // Output format for spectrum data
    ) -> SpectrumAccumulator {
        SpectrumAccumulator {
            acc: vec![0.0; if complex { fft_size } else { fft_size/2+1 }],
            accn: 0,
            fft_size: fft_size,
            averages: averages,
            outfmt: outfmt,
        }
    }

    pub fn accumulate(
        &mut self,
        fft_results: &[&mut[Complex<f32>]],
        ) -> std::io::Result<()>
    {
        for fft_result in fft_results.iter() {
            let fft_size = self.fft_size;
            let getbin = |i: isize| get_bin(fft_result, fft_size, i);

            // rayon seems to slow it down here. Maybe parallelization could be made optional.
            // For now, rayon is not used.
            self.acc.iter_mut().enumerate().for_each(
                |(i, acc_bin)|
            {
                /* Perform convolution in frequency domain with -0.5, 1, -0.5,
                 * equivalent to applying a Hann window before the FFT */
                let c = getbin(i as isize)
                      -(getbin(i as isize - 1) +
                        getbin(i as isize + 1)) * 0.5;

                *acc_bin += c.re * c.re + c.im * c.im;
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
