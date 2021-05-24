use std;
use std::io::Read;
use byte::*;
use rustfft::num_complex::Complex;
//use rayon::prelude::*;

#[macro_use]
extern crate clap;

mod dsp;
mod multifft;


fn parse_configuration() -> (dsp::DspParams, InputFormat) {
    use clap::{App};
    let matches = App::new("spektri")
        .args_from_usage("
            -s, --fftsize=[SIZE]             'FFT size'
            -b, --fftbuf=[NUMBER]            'Number of FFTs in each input buffer (adjust to optimize performance)'
            -a, --averages=[NUMBER]          'Number of FFTs averaged for spectrum'
            -f, --inputformat=[FORMAT]       'Input signal format'
            -F, --spectrumformat=[FORMAT]    'Spectrum output format'
            ")
        .get_matches();

    let inputformat = value_t!(matches, "inputformat", InputFormat).unwrap_or_else(|e| e.exit());

    (dsp::DspParams {
        complex:
            is_input_format_complex(inputformat),
        fft_size:
            value_t!(matches, "fftsize", usize)
            .unwrap_or(16384),
        scaling:
            input_format_scaling(inputformat),
        ffts_per_buf:
            value_t!(matches, "fftbuf", usize)
            .unwrap_or(8),
        spectrum_format:
            value_t!(matches, "inputformat", SpectrumFormat)
            .unwrap_or(SpectrumFormat::U8)
            .into(),
        spectrum_averages:
            value_t!(matches, "averages", u32)
            .unwrap_or(2000),
    },
    inputformat)
}


fn main() -> std::io::Result<()> {
    let (dspparams, inputformat) = parse_configuration();

    if is_input_format_complex(inputformat) {
        mainloop_complex(dspparams, inputformat)
    } else {
        mainloop_real(   dspparams, inputformat)
    }?;
    Ok(())
}


fn mainloop_complex(dspparams: dsp::DspParams, fmt: InputFormat) -> std::io::Result<()> {
    let (mut dsp, bufsize) = dsp::DspState::init(dspparams);

    // buffer for raw input data
    let mut rawbuf: Vec<u8> = vec![0; bytes_per_input_sample(fmt) * bufsize.new];

    // buffer for type converted data with overlap
    let mut buf: Vec<Complex<f32>> = vec![Complex{re:0.0,im:0.0}; bufsize.total ];

    let mut input = std::io::stdin();
    'mainloop: loop {
        // copy the overlapping part to beginning of the buffer
        buf.copy_within(bufsize.new .. bufsize.total, 0);

        // read input samples, type convert and write to the rest of the buffer
        match input.read_exact(&mut rawbuf) {
            Err (_) => { break 'mainloop; }
            Ok  (_) => { }
        }
        convert_to_cf32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total], fmt);

        dsp.process_complex(&buf)?;
    }
    Ok(())
}

fn mainloop_real(dspparams: dsp::DspParams, fmt: InputFormat) -> std::io::Result<()> {
    let (mut dsp, bufsize) = dsp::DspState::init(dspparams);

    // buffer for raw input data
    let mut rawbuf: Vec<u8> = vec![0; bytes_per_input_sample(fmt) * bufsize.new];

    // buffer for type converted data with overlap
    let mut buf: Vec<f32> = vec![0.0; bufsize.total];

    let mut input = std::io::stdin();
    'mainloop: loop {
        // copy the overlapping part to beginning of the buffer
        buf.copy_within(bufsize.new .. bufsize.total, 0);

        // read input samples, type convert and write to the rest of the buffer
        match input.read_exact(&mut rawbuf) {
            Err (_) => { break 'mainloop; }
            Ok  (_) => { }
        }
        convert_to_f32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total], fmt);

        dsp.process_real(&buf)?;
    }
    Ok(())
}


/*
 * Input formats and conversion functions
 */


arg_enum! {
    #[derive(Debug, Copy, Clone)]
    enum InputFormat {
        U8,      // real unsigned 8-bit
        S8,      // real signed 8-bit
        S16le,   // real signed 16-bit, little endian
        S16be,   // real signed 16-bit, big endian
        F32le,   // real float 32-bit, little endian
        F32be,   // real float 32-bit, big endian
        Cu8,     // complex unsigned 8-bit
        Cs8,     // complex signed 8-bit
        Cs16le,  // complex signed 16-bit, little endian
        Cs16be,  // complex signed 16-bit, big endian
        Cf32le,  // complex float 32-bit, little endian
        Cf32be,  // complex float 32-bit, big endian
    }
}

fn bytes_per_input_sample(fmt: InputFormat) -> usize {
    match fmt {
        InputFormat::U8     => 1,
        InputFormat::S8     => 1,
        InputFormat::S16le  => 2,
        InputFormat::S16be  => 2,
        InputFormat::F32le  => 4,
        InputFormat::F32be  => 4,
        InputFormat::Cu8    => 2,
        InputFormat::Cs8    => 2,
        InputFormat::Cs16le => 4,
        InputFormat::Cs16be => 4,
        InputFormat::Cf32le => 8,
        InputFormat::Cf32be => 8,
    }
}

fn is_input_format_complex(fmt: InputFormat) -> bool {
    match fmt {
        InputFormat::U8     |
        InputFormat::S8     |
        InputFormat::S16le  |
        InputFormat::S16be  |
        InputFormat::F32le  |
        InputFormat::F32be  => false,
        InputFormat::Cu8    |
        InputFormat::Cs8    |
        InputFormat::Cs16le |
        InputFormat::Cs16be |
        InputFormat::Cf32le |
        InputFormat::Cf32be => true,
    }
}

/* Scaling factors for each input format to get numbers between -1 and 1 */
fn input_format_scaling(fmt: InputFormat) -> f32 {
    match fmt {
        InputFormat::U8     |
        InputFormat::S8     |
        InputFormat::Cu8    |
        InputFormat::Cs8    => 1.0 / std::i8::MAX as f32,
        InputFormat::S16le  |
        InputFormat::S16be  |
        InputFormat::Cs16le |
        InputFormat::Cs16be => 1.0 / std::i16::MAX as f32,
        InputFormat::F32le  |
        InputFormat::F32be  |
        InputFormat::Cf32le |
        InputFormat::Cf32be => 1.0,
    }
}


fn convert_to_f32(src: &[u8], dst: &mut [f32], fmt: InputFormat) {
    let mut offset = 0;
    match fmt {
        InputFormat::U8 =>
        for v in dst.iter_mut() {
            *v = src.read_with::<u8>(&mut offset, LE).unwrap() as f32 - 127.4;
        },

        InputFormat::S8 =>
        for v in dst.iter_mut() {
            *v = src.read_with::<i8>(&mut offset, LE).unwrap() as f32;
        },

        InputFormat::S16le =>
        for v in dst.iter_mut() {
            *v = src.read_with::<i16>(&mut offset, LE).unwrap() as f32;
        },

        InputFormat::S16be =>
        for v in dst.iter_mut() {
            *v = src.read_with::<i16>(&mut offset, BE).unwrap() as f32;
        },

        InputFormat::F32le =>
        for v in dst.iter_mut() {
            *v = src.read_with::<f32>(&mut offset, LE).unwrap();
        },

        InputFormat::F32be =>
        for v in dst.iter_mut() {
            *v = src.read_with::<f32>(&mut offset, BE).unwrap();
        },

        _ => panic!("real conversion called with complex format parameter") // bug somewhere
    }
}

fn convert_to_cf32(src: &[u8], dst: &mut [Complex<f32>], fmt: InputFormat) {
    let mut offset = 0;
    match fmt {
        InputFormat::Cu8 =>
        for v in dst.iter_mut() {
            *v = Complex{
                re: src.read_with::<u8>(&mut offset, LE).unwrap() as f32 - 127.4,
                im: src.read_with::<u8>(&mut offset, LE).unwrap() as f32 - 127.4
            }
        },

        InputFormat::Cs8 =>
        for v in dst.iter_mut() {
            *v = Complex{
                re: src.read_with::<i8>(&mut offset, LE).unwrap() as f32,
                im: src.read_with::<i8>(&mut offset, LE).unwrap() as f32
            }
        },

        InputFormat::Cs16le =>
        for v in dst.iter_mut() {
            *v = Complex{
                re: src.read_with::<i16>(&mut offset, LE).unwrap() as f32,
                im: src.read_with::<i16>(&mut offset, LE).unwrap() as f32
            }
        },

        InputFormat::Cs16be =>
        for v in dst.iter_mut() {
            *v = Complex{
                re: src.read_with::<i16>(&mut offset, BE).unwrap() as f32,
                im: src.read_with::<i16>(&mut offset, BE).unwrap() as f32
            }
        },

        InputFormat::Cf32le =>
        for v in dst.iter_mut() {
            *v = Complex{
                re: src.read_with::<f32>(&mut offset, LE).unwrap(),
                im: src.read_with::<f32>(&mut offset, LE).unwrap()
            }
        },

        InputFormat::Cf32be =>
        for v in dst.iter_mut() {
            *v = Complex{
                re: src.read_with::<f32>(&mut offset, BE).unwrap(),
                im: src.read_with::<f32>(&mut offset, BE).unwrap()
            }
        },

        _ => panic!("complex conversion called with real format parameter") // bug somewhere
    }
}


/* Define a separate SpectrumFormat enum here
 * to avoid depending on clap in the dsp module. */

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    enum SpectrumFormat {
        U8,
        U16
    }
}

impl From<SpectrumFormat> for dsp::SpectrumFormat {
    fn from(fmt: SpectrumFormat) -> Self {
        match fmt {
            SpectrumFormat::U8  => dsp::SpectrumFormat::U8,
            SpectrumFormat::U16 => dsp::SpectrumFormat::U16,
        }
    }
}
