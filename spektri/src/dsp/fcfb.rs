//! Fast-convolution filter bank

use std::error::Error;
use rayon::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use zmq;

use super::data::*;
use super::fftutil::*;
use super::output::*;
use super::Metadata;

// ------------------------------------------------------
// Filter bank, code to combine multiple filter instances
// ------------------------------------------------------

/// Bank of filters
pub struct Fcfb {
    fft_info: FftInfo,
    filters: Vec<Filter>,
}

/// One filter
pub struct Filter {
    dsp: FilterDsp,
    outbuf: Vec<u8>,
    outsize: usize,
    output: Output,
}

impl Fcfb {
    pub fn init(
        fft_info: FftInfo,
    ) -> Self {
        Self {
            fft_info: fft_info,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(
        &mut self,
        p: &FilterParams,
    )
    {
        if let Some(bn) = freq_to_bins_exact(self.fft_info, p.fs_out, p.fc_out) {
            match FilterDsp::init(self.fft_info.size, bn) {
                Ok(filter) => self.filters.push(Filter {
                    dsp: filter,
                    // TODO: calculate sufficient size for the output buffer
                    outbuf: vec![0; 100000],
                    outsize: 0,
                    output: Output::init(&p.output, &serialize_signal_topic(&SignalInfo {
                        fs: p.fs_out,
                        fc: p.fc_out,
                    })),
                }),
                Err(error) => eprintln!("Error creating filter: {:?}", error),
            }
        } else {
            eprintln!("Invalid filter parameters");
            // TODO: return errors
        }
    }

    pub fn process(
        &mut self,
        fft_results: &[&mut[Complex<f32>]],
        metadata: &Metadata,
        sock: &zmq::Socket, // ZeroMQ socket used to publish all results
    )
    {
        // Process multiple filters in parallel
        self.filters.par_iter_mut().for_each( |filter| {
            let mut offset = 0;
            // Sequence number of can be the same as metadata.seq because
            // one record is produced for each processing block.
            // This also results in common sequence numbering for all filters
            // which is convenient for applications requiring
            // synchronized signals from multiple filters.
            //
            // unwrap is OK here because it would only panic if outbuf
            // is too small for metadata. That would clearly be a bug.
            serialize_metadata(&mut filter.outbuf, &mut offset, &metadata, metadata.seq).unwrap();
            for fft_result in fft_results.iter() {
                if filter.dsp.done { break; }
                filter.dsp.process(fft_result, &mut filter.outbuf, &mut offset);
            }
            filter.outsize = offset;
        });

        // Do I/O outside of the parallel part.
        self.filters.iter_mut().for_each( |filter| {
            filter.output.write(&filter.outbuf[0..filter.outsize], sock);
        });

        // Remove filters that are done
        // (not actually used for anything at the moment)
        self.filters.retain(|f| !f.dsp.done);
    }

    /// Calculate the nearest possible exact sample rate and center frequency
    /// for a filter output.
    ///
    /// Return a tuple of (sample rate, center frequency).
    /// These values will be accepted by add_filter.
    /// If the values would be impossible, return None.
    pub fn nearest_freq(
        &self,
        fs_out: f64,
        fc_out: f64,
    ) -> Option<(f64, f64)> {
        let bn = freq_to_bins(self.fft_info, fs_out, fc_out)?;
        Some(bins_to_freq(self.fft_info, bn))
    }
}



// ----------------------------------------------
// Signal processing for a single filter instance
// ----------------------------------------------

/// DSP state of a filter.
///
/// This struct does not contain any I/O related things. */
pub struct FilterDsp {
    // TODO: is done useful anymore?
    done: bool,
    fft_size: usize,
    freq: isize,
    weights: Vec<f32>, // Frequency response
    ifft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
}

impl FilterDsp {
    pub fn init(
        fft_size: usize,
        bn:       BinNumbers,
    ) -> Result<Self, Box<dyn Error>>
    // TODO: Box<dyn Error> is not used here anymore, so use something else
    {
        // TODO: reuse the planner
        let mut planner = FftPlanner::new();
        Ok(Self {
            done: false,
            fft_size: fft_size,
            freq: bn.first,
            weights: raised_cosine_weights(bn.bins),
            ifft: planner.plan_fft_inverse(bn.bins),
        })
    }

    pub fn process(
        &mut self,
        fft_result: &[Complex<f32>],
        outbuf: &mut [u8],
        outbuf_offset: &mut usize,
    ) {
        let fft_size = self.fft_size;
        let ifft_size = self.ifft.len();
        let freq = self.freq;

        let ifft_shift = ifft_size / 2;

        let mut buf: Vec<Complex<f32>> =
        (0..ifft_size).map(|i|
            (i + ifft_shift) % ifft_size
        ).map(|i|
            self.weights[i] *
            get_bin(fft_result, fft_size, freq + (i as isize))
        ).collect();

        self.ifft.process(&mut buf);

        // fixed 25% overlap
        let result = &buf[ifft_size / 8 .. ifft_size / 8 * 7];

        // Write result to output buffer
        use byte::*;
        for v in result.into_iter() {
            // unwrap will panic if outbuf is too small.
            // This may actually happen because the buffer size
            // is not properly calculated yet.
            outbuf.write_with(outbuf_offset, v.re, LE).unwrap();
            outbuf.write_with(outbuf_offset, v.im, LE).unwrap();
        }
    }
}



// -------------
// Filter design
// -------------

fn raised_cosine_weights(size: usize) -> Vec<f32> {
    use std::f32::consts::PI;
    let f = (2.0 * PI) / size as f32;
    (0..size).map(|i| {
        0.5 - 0.5 * ((i as f32) * f).cos()
    }).collect()
}

/// Filter design and configuration parameters
pub struct FilterParams {
    pub fs_out: f64, // Output sample rate
    pub fc_out: f64, // Output center frequency
    pub output: OutputParams,
}

#[derive(Copy, Clone)]
pub struct BinNumbers {
    pub bins:  usize, // IFFT size, number of bins to use
    pub first: isize, // Number of the first bin
}

/// Convert sample rate and center frequency to bin numbers for a filter.
///
/// Values will be rounded to the nearest possible one.
/// Return none if the values would be impossible.
///
/// To get the exact frequency values, use bins_to_freq
/// to convert the numbers back to frequencies.
pub fn freq_to_bins(
    fft_info: FftInfo,
    fs_out:   f64,   // Filtered sample rate
    fc_out:   f64,   // Filtered center frequency
) -> Option<BinNumbers> {
    // IFFT size must be a multiple of this.
    // For 25% overlap, it's 4. For 50% overlap (TODO), it would be 2.
    let multiple: isize = 4;
    fn nearest_multiple(value: f64, multiple: isize) -> isize {
        ((value / (multiple as f64)).round() * (multiple as f64)) as isize
    }

    let bin_spacing = fft_info.fs / (fft_info.size as f64);

    // IFFT size
    let bins = nearest_multiple(fs_out / bin_spacing, multiple);
    // Center bin number
    let center = nearest_multiple((fc_out - fft_info.fc) / bin_spacing, multiple);

    if bins > 0 { // TODO: other validity checks?
        Some(BinNumbers {
            bins:  bins as usize,
            first: center - bins/2
        })
    } else {
        None
    }
}

/// Convert bin numbers to the resulting sample rate and center frequency.
///
/// Return a tuple of the output sample rate and output center frequency.
pub fn bins_to_freq(
    fft_info: FftInfo,
    bn:       BinNumbers,
) -> (f64, f64) {
    let bin_spacing = fft_info.fs / (fft_info.size as f64);
    (
        // Output sample rate
        bin_spacing * (bn.bins as f64),
        // Output center frequency
        fft_info.fc + bin_spacing * ((bn.first + (bn.bins/2) as isize) as f64)
    )
}


/// Convert sample rate and center frequency to bin numbers for a filter.
/// Only accept frequency values that can be done exactly.
/// Return none if the values would be impossible.
pub fn freq_to_bins_exact(
    fft_info: FftInfo,
    fs_out:   f64,   // Filtered sample rate
    fc_out:   f64,   // Filtered center frequency
) -> Option<BinNumbers> {
    let bn = freq_to_bins(fft_info, fs_out, fc_out)?;
    if bins_to_freq(fft_info, bn) == (fs_out, fc_out) {
        Some(bn)
    } else {
        None
    }
}


#[test]
fn test_freq_to_bins() {
    fn test(
        fft_size: usize,
        fs_in:    f64,
        fc_in:    f64,
        fs_out:   f64,
        fc_out:   f64,
        bins:     usize, // Expected result
        first:    isize, // Expected result
        should_be_exact: bool,
    ) {
        let fft_info = FftInfo { fs: fs_in, fc: fc_in, size: fft_size, complex: true };
        let bn = freq_to_bins(fft_info, fs_out, fc_out).unwrap();
        assert!(bn.bins == bins);
        assert!(bn.first == first);
        let (fs_out_exact, fc_out_exact) = bins_to_freq(fft_info, bn);
        if should_be_exact {
            assert!(fs_out_exact == fs_out);
            assert!(fc_out_exact == fc_out);
        }
    }
    // Test with some values that were used before
    test(16384, 128.0e6, 0.0, 500000.0, 50.250e6, 64, 6400, true);
}
