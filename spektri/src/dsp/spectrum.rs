/*
 * Spectrum analysis
 *
 * The code here takes FFT results and turns them into
 * spectrum analysis results.
 *
 * The squared magnitude of each FFT bin is averaged over a number of FFTs.
 * Result of that is then quantized on a logarithmic scale and written
 * into the output file.
 */

use rustfft::num_complex::Complex;
use std::io::Write;


arg_enum! { // needed for command line parsing
    #[derive(Debug, Copy, Clone)]
    pub enum SpectrumFormat { U8, U16 }
}

pub struct SpectrumAccumulator {
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
    averages: u32, // parameter
    outfmt: SpectrumFormat, // parameter
}

impl SpectrumAccumulator {
    pub fn init(
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

    pub fn accumulate(
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
