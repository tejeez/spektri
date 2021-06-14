/* Fast-convolution filter bank */

use std::fs::File;
use std::io::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use super::fftutil::*;

/* Bank of filters */
pub struct Fcfb {
    f: FcFilter, // TODO: some data structure to hold multiple filters
}

impl Fcfb {
    pub fn init(
        fft_size: usize,
    ) -> Self {
        Self {
            f: FcFilter::init(fft_size, 32),
        }
    }

    pub fn process(
        &mut self,
        fft_results: &[&mut[Complex<f32>]],
    )
    {
        for fft_result in fft_results.iter() {
            self.f.process(fft_result);
        }
    }
}


/* One filter */
pub struct FcFilter {
    fft_size: usize,
    ifft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    output_file: File,
}

impl FcFilter {
    pub fn init(
        fft_size: usize,
        ifft_size: usize,
    ) -> Self {
        // TODO: reuse the planner
        let mut planner = FftPlanner::new();
        Self {
            fft_size: fft_size,
            ifft: planner.plan_fft_inverse(ifft_size),
            // Proper error handling is missing here, but the file output
            // implemented for now is intended only for initial testing anyway
            output_file: File::create("data/filtered").unwrap(),
        }
    }

    pub fn process(
        &mut self,
        fft_result: &[Complex<f32>],
    )
    {
        let fft_size = self.fft_size;
        let ifft_size = self.ifft.len();
        let freq: isize = 100;

        let ifft_shift = ifft_size / 2;

        let mut buf: Vec<Complex<f32>> =
        (0..ifft_size).map(|i|
            (i + ifft_shift) % ifft_size
        ).map(|i|
            // TODO: filter coefficients
            get_bin(fft_result, fft_size, freq + (i as isize))
        ).collect();

        self.ifft.process(&mut buf);

        // fixed 25% overlap
        write_to_file(&mut self.output_file, &buf[ifft_size / 8 .. ifft_size / 8 * 7]);
    }
}


fn write_to_file(f: &mut File, data: &[Complex<f32>]) {
    use byte::*;
    let mut outbuf: Vec<u8> = vec![0; data.len() * 8];
    let mut o = 0;
    for v in data.into_iter() {
        outbuf.write_with(&mut o, v.re, LE);
        outbuf.write_with(&mut o, v.im, LE);
    }
    f.write_all(&outbuf).unwrap();
}
