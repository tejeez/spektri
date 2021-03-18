use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;
use std::io::Write;

pub struct DspState {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>, // RustFFT plan
    acc: Vec<f32>, // Accumulator for FFT averaging
    accn: u32, // Counter for number of FFTs averaged
    window: Vec<f32>, // Window function
}

impl DspState {
    pub fn init(fftsize: usize, scaling: f32) -> DspState {
        let mut planner = FftPlanner::new();
        DspState {
            fft: planner.plan_fft_forward(fftsize),
            acc: vec![0.0; fftsize],
            accn: 0,
            window: {
                let mut w = vec![0.0; fftsize];
                let mut s: f32 = 0.0;
                // Hann window
                for i in 0..fftsize {
                    w[i] = 1.0 - ((i as f32 + 0.5) * 2.0 * PI / fftsize as f32).cos();
                    s += w[i]; // calculate the sum for normalization
                }
                // Include all the scaling factors into the window function.
                // Normalize sum over the whole window to the desired scaling.
                s = scaling / s;
                for i in 0..fftsize {
                    w[i] *= s;
                }
                w
            },
        }
    }

    pub fn process(&mut self, input_buffer: &[Complex<f32>]) -> std::io::Result<()> {
        // Buffer for FFT calculation
        let mut fft_buffer: Vec<Complex<f32>> = vec![Complex{re:0.0, im:0.0}; input_buffer.len()];
        // Apply the window function
        for (buffer, (input, window)) in fft_buffer.iter_mut().zip(input_buffer.iter().zip(self.window.iter())) {
            *buffer = Complex{
                re: input.re * window,
                im: input.im * window
            };
        }

        self.fft.process(&mut fft_buffer);
        for (fft_bin, acc_bin) in fft_buffer.iter().zip(self.acc.iter_mut()) {
            *acc_bin += fft_bin.re * fft_bin.re + fft_bin.im * fft_bin.im;
        }
        self.accn += 1;
        let averages = 100;
        if self.accn >= averages {
            // Write the result in binary format into stdout

            let mut printbuf: Vec<u8> = vec![0; self.acc.len()];

            // divide accumulator bins by self.accn,
            // but do it as an addition after conversion to dB scale
            let db_plus = (self.accn as f32).log10() * -10.0;

            for (acc_bin, out) in self.acc.iter().zip(printbuf.iter_mut()) {
                let db = acc_bin.log10() * 10.0 + db_plus;
                // quantize to 0.5 dB per LSB, full scale at 250
                *out = (db * 2.0 + 250.0).max(0.0).min(255.0) as u8;
            }
            std::io::stdout().write_all(&printbuf)?;

            // Reset accumulator
            for acc_bin in self.acc.iter_mut() {
                *acc_bin = 0.0;
            }
            self.accn = 0;
        }
        Ok(())
    }
}

