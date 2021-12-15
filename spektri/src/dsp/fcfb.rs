/* Fast-convolution filter bank */

use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use rayon::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};
use super::fftutil::*;

/* Bank of filters */
pub struct Fcfb {
    fft_size: usize,
    filters: Vec<FcFilter>,
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

    // temporary init function for initial testing
    pub fn init_test(
        fft_size: usize,
    ) -> Self
    {
        let mut s = Self::init(fft_size);
        s.add_filter(32, 100, "data/filtered1");
        s.add_filter(8, 200, "data/filtered2");
        s
    }

    pub fn add_filter(
        &mut self,
        ifft_size: usize,
        freq: isize,
        filename: &str,
    )
    {
        match FcFilter::init(self.fft_size, ifft_size, freq, filename) {
            Ok(filter) => self.filters.push(filter),
            Err(error) => eprintln!("Error creating filter: {:?}", error),
        }
    }

    pub fn process(
        &mut self,
        fft_results: &[&mut[Complex<f32>]],
    )
    {
        self.filters.par_iter_mut().for_each( |filter| {
            for fft_result in fft_results.iter() {
                if filter.done { break; }
                match filter.process(fft_result) {
                    Err(error) => {
                        eprintln!("Error running filter: {:?}", error);
                        // Mark failed filter as done, so it gets removed
                        filter.done = true;
                    },
                    Ok(_) => {}
                }
            }
        });
        // Remove filters that are done
        self.filters.retain(|f| !f.done);
    }
}


/* Filter parameters */
pub struct FilterParams/*<'a>*/ {
    pub freq: isize,  // Center frequency
    pub ifft_size: usize,
    //pub bw: isize, // Bandwidth
    pub filename: String,
}

/* One filter */
pub struct FcFilter {
    done: bool,
    fft_size: usize,
    freq: isize,
    weights: Vec<f32>, // Frequency response
    ifft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    output_file: File,
}

impl FcFilter {
    pub fn init(
        fft_size: usize,
        ifft_size: usize,
        freq: isize, // Center frequency
        filename: &str,
    ) -> Result<Self, Box<dyn Error>>
    {
        // TODO: reuse the planner
        let mut planner = FftPlanner::new();
        Ok(Self {
            done: false,
            fft_size: fft_size,
            freq: freq,
            weights: raised_cosine_weights(ifft_size),
            ifft: planner.plan_fft_inverse(ifft_size),
            output_file: File::create(filename)?,
        })
    }

    pub fn process(
        &mut self,
        fft_result: &[Complex<f32>],
    ) -> Result<(), Box<dyn Error>>
    {
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
        write_to_file(&mut self.output_file, &buf[ifft_size / 8 .. ifft_size / 8 * 7])
    }
}


fn write_to_file(f: &mut File, data: &[Complex<f32>])
-> Result<(), Box<dyn Error>>
{
    use byte::*;
    let mut outbuf: Vec<u8> = vec![0; data.len() * 8];
    let mut o = 0;
    for v in data.into_iter() {
        outbuf.write_with(&mut o, v.re, LE);
        outbuf.write_with(&mut o, v.im, LE);
    }
    f.write_all(&outbuf)?;
    Ok(())
}

fn raised_cosine_weights(size: usize) -> Vec<f32> {
    use std::f32::consts::PI;
    let f = (2.0 * PI) / size as f32;
    (0..size).map(|i| {
        0.5 - 0.5 * ((i as f32) * f).cos()
    }).collect()
}
