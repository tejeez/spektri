use rustfft::num_complex::Complex;
use std::f32::consts::PI;
use zmq;

mod multifft;
use multifft::MultiFft;

mod spectrum;
use spectrum::SpectrumAccumulator;
pub use spectrum::SpectrumFormat;

mod fcfb;
use fcfb::Fcfb;
pub use fcfb::FilterParams;

pub mod fftutil;
pub mod output;


// Parameters for signal processing
pub struct DspParams {
    pub complex: bool, // Type of input signal: true for I/Q, false for real
    pub fft_size: usize,
    pub scaling: f32, // Scaling of input values
    pub ffts_per_buf: usize,
    //pub fft_overlap: usize, // Fixed for now
    pub spectrum_format: SpectrumFormat, // Output format for spectrum data
    pub spectrum_averages: u32, // Number of FFTs averaged
    pub filters: Vec<FilterParams>, // Filter bank parameters
}

// requirements for the buffers given to DspState::process
pub struct InputBufferSize {
    pub new:     usize, // Number of new samples in each buffer
    pub overlap: usize, // Overlampping samples from previous buffer
    pub total:   usize, // Total size of the buffer (new+overlap)
}

// metadata for buffers of samples
pub struct Metadata {
    pub systemtime: std::time::SystemTime,
}

pub struct DspState {
    fft_size: usize,
    ffts_per_buf: usize,
    fft_interval: usize,

    mfft: MultiFft,
    accu: SpectrumAccumulator,
    fb: Fcfb,

    window: Vec<f32>, // Window function,
    fft_result_buf: Vec<Complex<f32>>, // Pre-allocated buffer
}

fn hann_window(size: usize, scaling: f32) -> Vec<f32> {
    let mut w: Vec<f32> = (0..size).map(|i| {
        1.0 - ((i as f32 + 0.5) * 2.0 * PI / size as f32).cos()
    }).collect();

    // Include all the scaling factors into the window function.
    // Normalize sum over the whole window to the desired scaling.
    let s: f32 = scaling / w.iter().sum::<f32>();
    w.iter_mut().for_each(|v| { *v *= s; });

    w
}

fn rectangular_window(size: usize, scaling: f32) -> Vec<f32> {
    let s = scaling / (size as f32);
    (0..size).map(|_i| s).collect()
}

impl DspState {
    pub fn init(params: DspParams) -> (DspState, InputBufferSize) {
        let fft_overlap = params.fft_size / 4; // 25% overlap
        let fft_interval = params.fft_size - fft_overlap; // FFT is taken every fft_interval samples
        let result_bins = if params.complex { params.fft_size } else { params.fft_size / 2 + 1 };

        (DspState {
            fft_size: params.fft_size,
            ffts_per_buf: params.ffts_per_buf,
            fft_interval: fft_interval,

            mfft: MultiFft::init(params.fft_size),
            accu: SpectrumAccumulator::init(params.fft_size, params.complex, params.spectrum_averages, params.spectrum_format),
            fb: {
                let mut fb = Fcfb::init(params.fft_size);
                for f in params.filters.iter() {
                    fb.add_filter(f);
                }
                fb
            },

            // TODO: Now that a rectangular window is used,
            // consider removing the multiplication with a window function
            // and just replacing it with a constant scaling.
            window: rectangular_window(params.fft_size, params.scaling),
            fft_result_buf: vec![Complex{re:0.0, im:0.0}; result_bins * params.ffts_per_buf],
        }, InputBufferSize {
            overlap: fft_overlap,
            new: fft_interval * params.ffts_per_buf,
            total: fft_overlap + fft_interval * params.ffts_per_buf
        })
    }

    pub fn process_complex(
        &mut self,
        input_buffer: &[Complex<f32>],
        metadata: &Metadata,
        sock: &zmq::Socket,
    ) -> std::io::Result<()> {
        let fft_interval = self.fft_interval;
        let fft_size = self.fft_size;

        // Buffers for FFT results
        let mut resultbufs: Vec<&mut [Complex<f32>]> = self.fft_result_buf.chunks_mut(fft_size).collect();

        self.mfft.process_complex(
            &self.window,
            &(0..self.ffts_per_buf).map(|i|
                &input_buffer[i*fft_interval .. i*fft_interval+fft_size]
            ).collect::<Vec<&[Complex<f32>]>>(),
            &mut resultbufs
        );
        self.accu.accumulate(&resultbufs, metadata, sock)?;
        self.fb.process(&resultbufs, metadata, sock);
        Ok(())
    }

    pub fn process_real(
        &mut self,
        input_buffer: &[f32],
        metadata: &Metadata,
        sock: &zmq::Socket,
    ) -> std::io::Result<()> {
        let fft_interval = self.fft_interval;
        let fft_size = self.fft_size;

        // Buffers for FFT results
        let mut resultbufs: Vec<&mut [Complex<f32>]> = self.fft_result_buf.chunks_mut(fft_size/2+1).collect();

        self.mfft.process_real(
            &self.window,
            &(0..self.ffts_per_buf).map(|i|
                &input_buffer[i*fft_interval .. i*fft_interval+fft_size]
            ).collect::<Vec<&[f32]>>(),
            &mut resultbufs
        );
        self.accu.accumulate(&resultbufs, metadata, sock)?;
        self.fb.process(&resultbufs, metadata, sock);
        Ok(())
    }
}
