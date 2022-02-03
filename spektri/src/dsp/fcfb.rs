/* Fast-convolution filter bank */

use std::error::Error;
use rayon::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use zmq;

use super::fftutil::*;
use super::output::*;
use super::Metadata;

/* Bank of filters */
pub struct Fcfb {
    fft_size: usize,
    filters: Vec<Filter>,
}

/* One filter */
pub struct Filter {
    dsp: FilterDsp,
    outbuf: Vec<u8>,
    outsize: usize,
    output: Output,
}

impl Fcfb {
    pub fn init(
        fft_size: usize,
    ) -> Self {
        Self {
            fft_size: fft_size,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(
        &mut self,
        p: &FilterParams,
    )
    {
        match FilterDsp::init(self.fft_size, &p) {
            Ok(filter) => self.filters.push(Filter {
                dsp: filter,
                // TODO: calculate sufficient size for the output buffer
                outbuf: vec![0; 100000],
                outsize: 0,
                output: Output::init(&p.output),
            }),
            Err(error) => eprintln!("Error creating filter: {:?}", error),
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
            // TODO sample sequence numbers
            serialize_metadata(&mut filter.outbuf, &mut offset, &metadata, 0);
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
}


/* Filter parameters */
pub struct FilterParams {
    pub freq: isize,  // Center frequency
    pub ifft_size: usize,
    //pub bw: isize, // Bandwidth
    pub output: OutputParams,
}

/* DSP state of a filter.
 * This struct does not contain any I/O related things. */
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
        p: &FilterParams,
    ) -> Result<Self, Box<dyn Error>>
    // TODO: Box<dyn Error> is not used here anymore, so use something else
    {
        // TODO: reuse the planner
        let mut planner = FftPlanner::new();
        Ok(Self {
            done: false,
            fft_size: fft_size,
            freq: p.freq,
            weights: raised_cosine_weights(p.ifft_size),
            ifft: planner.plan_fft_inverse(p.ifft_size),
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
            outbuf.write_with(outbuf_offset, v.re, LE);
            outbuf.write_with(outbuf_offset, v.im, LE);
        }
    }
}


fn raised_cosine_weights(size: usize) -> Vec<f32> {
    use std::f32::consts::PI;
    let f = (2.0 * PI) / size as f32;
    (0..size).map(|i| {
        0.5 - 0.5 * ((i as f32) * f).cos()
    }).collect()
}
