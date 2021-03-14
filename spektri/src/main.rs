use std;
use std::io::Read;
use std::io::Write;
use byte::*;
use rustfft::{FftPlanner, num_complex::Complex};

struct DspState {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
}

impl DspState {
    fn init(fftsize: usize) -> DspState {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fftsize);
        return DspState {
            fft: fft,
            acc: vec![0.0; fftsize],
            accn: 0,
        };
    }

    fn process(&mut self, input_buffer: &Vec<Complex<f32>>) {
        // RustFFT needs a mutable buffer, so clone the data
        let mut fft_buffer = input_buffer.to_vec();
        // TODO: window function
        self.fft.process(&mut fft_buffer);
        for (fft_bin, acc_bin) in fft_buffer.iter().zip(self.acc.iter_mut()) {
            *acc_bin += fft_bin.re * fft_bin.re + fft_bin.im * fft_bin.im;
        }
        self.accn += 1;
        let averages = 10000;
        if self.accn >= averages {
            // Print the result as ASCII art
            let mut printbuf: Vec<u8> = vec![10; self.acc.len() + 1];
            for (acc_bin, out) in self.acc.iter().zip(printbuf.iter_mut()) {
                *out = (32.0 + acc_bin * 1.0e-15) as u8;
            }
            std::io::stdout().write_all(&printbuf).unwrap();

            // Reset accumulator
            for acc_bin in self.acc.iter_mut() {
                *acc_bin = 0.0;
            }
            self.accn = 0;
        }
    }
}

fn main() -> std::io::Result<()> {
    let buf_samples: usize = 64;
    let overlap_samples: usize = buf_samples / 4;

    let mut dsp: DspState = DspState::init(buf_samples);

    // buffer for raw input data
    let mut rawbuf: Vec<u8> = vec![0; 4 * (buf_samples - overlap_samples)];
    // buffer for type converted data with overlap
    let mut buf: Vec<Complex<f32>> = vec![Complex{re:0.0,im:0.0}; buf_samples ];

    let mut input = std::io::stdin();

    loop {
        // copy the overlapping part to beginning of the buffer
        buf.copy_within((buf_samples - overlap_samples) .. buf_samples, 0);

        // read input samples, type convert and write to the rest of the buffer
        input.read_exact(&mut rawbuf)?;
        let mut rawbuf_offset: usize = 0;
        for i in overlap_samples .. buf_samples {
            let sample_i = rawbuf.read_with::<i16>(&mut rawbuf_offset, LE).unwrap() as f32;
            let sample_q = rawbuf.read_with::<i16>(&mut rawbuf_offset, LE).unwrap() as f32;
            buf[i] = Complex{ re: sample_i, im: sample_q };
        }
        dsp.process(&buf);
    }
}
