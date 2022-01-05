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
use super::Metadata;

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
        metadata: &Metadata,
        ) -> std::io::Result<()>
    {
        for fft_result in fft_results.iter() {
            // Perform convolution in frequency domain with -0.5, 1, -0.5,
            // equivalent to applying a Hann window before the FFT.
            // As an optimization, call getbin only for the first and last bins
            // where modulo indexing needs to be handled in a special way.

            // Special cases of first and last bins
            let fft_size = self.fft_size;
            let getbin = |i: isize| get_bin(fft_result, fft_size, i);
            for &i in [ 0, self.acc.len()-1 ].iter() {
                let c = getbin(i as isize)
                      -(getbin(i as isize - 1) +
                        getbin(i as isize + 1)) * 0.5;
                self.acc[i] += c.re * c.re + c.im * c.im;
            }

            // Faster way to process the rest of the bins
            self.acc[1..].iter_mut().zip(fft_result.windows(3)).for_each(
                |(acc_bin, w)|
            {
                let c = w[1] - (w[0] + w[2]) * 0.5;
                *acc_bin += c.re * c.re + c.im * c.im;
            });

            // Count the number of FFTs accumulated
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
                write_record(&printbuf, metadata)?;

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

fn serialize_metadata(
    metadata: &Metadata,
) -> [u8; 12] {
    use byte::*;
    use std::time::UNIX_EPOCH;

    let mut s: [u8; 12] = [0; 12];
    let mut o = 0;

    let (secs, nanosecs) = match metadata.systemtime.duration_since(UNIX_EPOCH) {
        Ok(d) => { (d.as_secs(), d.subsec_nanos()) },
        // If duration_since fails, let's just write zeros there.
        // Maybe we don't really need to handle it in any special way.
        Err(_) => { (0,0) },
    };

    s.write_with(&mut o, secs, LE);
    s.write_with(&mut o, nanosecs, LE);

    s
}

fn write_record(
    //file: File,
    data: &[u8],
    metadata: &Metadata,
) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    stdout.write_all(&serialize_metadata(metadata))?;
    stdout.write_all(&data)?;
    Ok(())
}
