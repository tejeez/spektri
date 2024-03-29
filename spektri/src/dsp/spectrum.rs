//! Spectrum analysis
//!
//! The code here takes FFT results and turns them into
//! spectrum analysis results using the Welch's method.
//!
//! The squared magnitude of each FFT bin is averaged over a number of FFTs.
//! Result of that is then quantized on a logarithmic scale and written
//! into the output file.
//!
//! As a modification from usual Welch's method,
//! the multiplication with a window function before the FFT is replaced
//! by an equivalent circular convolution in frequency domain after the FFT.
//!
//! Normally, this would not be particularly useful, since a convolution
//! requires more multiplications than an equivalent multiplication
//! in the other domain.
//!
//! Here, however, it lets us use a rectangular FFT window, which works
//! better for a fast-convolution filter-bank, allowing the use of the same
//! FFT results for both a filter bank and spectrum analysis.

use rustfft::num_complex::Complex;
use zmq;
use super::fftutil::*;
use super::Metadata;
use super::output::*;
use super::data::*;

arg_enum! { // needed for command line parsing
    #[derive(Debug, Copy, Clone)]
    pub enum SpectrumFormat { U8, U16 }
}

pub struct SpectrumAccumulator {
    /// Sequence number, number of results produced
    seq: u64,
    /// Accumulator for FFT averaging
    acc: Vec<f32>,
    /// Counter for number of FFTs averaged
    accn: u32,

    /// Parameter: information about FFT results
    fft_info: FftInfo,
    /// Parameter: Number of FFTs averaged
    averages: u32,
    /// Parameter: Output format for spectrum data
    outfmt: SpectrumFormat,
    output: Output,
}

impl SpectrumAccumulator {
    pub fn init(
        fft_info: FftInfo,
        averages: u32, // Number of FFTs averaged
        outfmt: SpectrumFormat, // Output format for spectrum data
    ) -> Self {
        let spectrum_info = SpectrumInfo {
            // TODO: implement "FFT shifting" when the input signal is complex.
            // Fix f0 for that case.
            f0: fft_info.fc,
            // TODO: consider calculating spacing of FFT bins somewhere in one place.
            fd: fft_info.fs / (fft_info.size as f64),
        };

        Self {
            seq: 0,
            // TODO: consider calculating number of FFT bins somewhere in one place.
            acc: vec![0.0; if fft_info.complex { fft_info.size } else { fft_info.size/2+1 }],
            accn: 0,
            fft_info: fft_info,
            averages: averages,
            outfmt: outfmt,
            output: Output::init(
                &OutputParams {
                    // Temporary hack for compatibility with old test scripts:
                    filename: Some("/dev/stdout".to_string()),
                },
                &serialize_spectrum_topic(&spectrum_info)),
        }
    }

    pub fn accumulate(
        &mut self,
        fft_results: &[&mut[Complex<f32>]],
        metadata: &Metadata,
        sock: &zmq::Socket, // ZeroMQ socket used to publish all results
        ) -> std::io::Result<()>
    {
        for fft_result in fft_results.iter() {
            // Perform convolution in frequency domain with -0.5, 1, -0.5,
            // equivalent to applying a Hann window before the FFT.
            // As an optimization, call getbin only for the first and last bins
            // where modulo indexing needs to be handled in a special way.

            // Special cases of first and last bins
            let fft_size = self.fft_info.size;
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
                let outfmt = self.outfmt;
                let mut outbuf: Vec<u8> = vec![
                    0;
                    100 + // TODO correct size of sample metadata
                    self.acc.len() * match outfmt { SpectrumFormat::U16=>2, SpectrumFormat::U8=>1 }];
                let mut offset = 0;

                // unwrap is OK here because it would only panic if outbuf
                // is too small for metadata. That would clearly be a bug.
                serialize_metadata(&mut outbuf, &mut offset, &metadata, self.seq).unwrap();

                // divide accumulator bins by self.accn,
                // but do it as an addition after conversion to dB scale
                let db_plus = (self.accn as f32).log10() * -10.0;

                match outfmt {
                SpectrumFormat::U16 => {
                    for (acc_bin, out) in self.acc.iter().zip(outbuf[offset..].chunks_mut(2)) {
                        let db = acc_bin.log10() * 10.0 + db_plus;
                        // quantize to 0.05 dB per LSB, full scale at 4000, clamp to 12 bits
                        let o = (db * 20.0 + 4000.0).max(0.0).min(4095.0) as u16;
                        // This could be changed to little endian for consistency
                        // but for now fixing the 16-bit format isn't a high priority.
                        out[0] = (o >> 8) as u8;
                        out[1] = (o & 0xFF) as u8;
                        offset += 2; // TODO maybe just use byte crate here as well
                    }
                },
                SpectrumFormat::U8 => {
                    for (acc_bin, out) in self.acc.iter().zip(outbuf[offset..].iter_mut()) {
                        let db = acc_bin.log10() * 10.0 + db_plus;
                        // quantize to 0.5 dB per LSB, full scale at 250
                        *out = (db * 2.0 + 250.0).max(0.0).min(255.0) as u8;
                        offset += 1; // TODO maybe just use byte crate here as well
                    }
                }}
                self.output.write(&outbuf[0..offset], &sock);

                // Reset accumulator
                for acc_bin in self.acc.iter_mut() {
                    *acc_bin = 0.0;
                }
                self.accn = 0;
                self.seq += 1;
            }
        }
        Ok(())
    }
}
