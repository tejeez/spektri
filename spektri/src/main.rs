use std;
use std::io::Read;
use byte::*;
use rustfft::num_complex::Complex;
//use rayon::prelude::*;

#[macro_use]
extern crate clap;

mod dsp;
mod multifft;


arg_enum! {
    #[derive(Debug, Copy, Clone)]
    enum InputFormat {
        S16le,   // real signed 16-bit, little endian
        Cs16le,  // complex signed 16-bit, little endian
    }
}

fn bytes_per_input_sample(fmt: InputFormat) -> usize {
    match fmt {
        InputFormat::S16le  => 2,
        InputFormat::Cs16le => 4,
    }
}

fn is_input_format_complex(fmt: InputFormat) -> bool {
    match fmt {
        InputFormat::S16le  => false,
        InputFormat::Cs16le => true,        
    }
}


fn parse_configuration() -> (dsp::DspParams, InputFormat) {
    use clap::{App};
    let matches = App::new("spektri")
        .args_from_usage("
            -s, --fftsize=[SIZE]             'FFT size'
            -a, --averages=[NUMBER]          'Number of FFTs averaged for spectrum'
            -f, --inputformat=[FORMAT]       'Input signal format'
            -F, --spectrumformat=[FORMAT]    'Spectrum output format'
            ")
        .get_matches();

    let inputformat = value_t!(matches, "inputformat", InputFormat).unwrap_or_else(|e| e.exit());
    (dsp::DspParams {
        complex: is_input_format_complex(inputformat),
        fft_size: value_t!(matches, "fftsize", usize).unwrap_or(16384),
        scaling: 1.0 / std::i16::MAX as f32,
        ffts_per_buf: 8,
        spectrum_format: if matches.value_of("spectrumformat").unwrap_or("u8") == "u16" { dsp::SpectrumFormat::U16 } else { dsp::SpectrumFormat::U8 },
        spectrum_averages: value_t!(matches, "averages", u32).unwrap_or(2000),
    }, inputformat)
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


fn convert_to_f32(src: &[u8], dst: &mut [f32], fmt: InputFormat) {
    let mut offset = 0;
    match fmt {
        InputFormat::S16le =>
            for v in dst.iter_mut() {
                *v = src.read_with::<i16>(&mut offset, LE).unwrap() as f32;
            },
        _ => panic!("real conversion called with complex format parameter") // bug somewhere
    }
}

fn convert_to_cf32(src: &[u8], dst: &mut [Complex<f32>], fmt: InputFormat) {
    let mut offset = 0;
    match fmt {
        InputFormat::Cs16le =>
            for v in dst.iter_mut() {
                *v = Complex{
                    re: src.read_with::<i16>(&mut offset, LE).unwrap() as f32,
                    im: src.read_with::<i16>(&mut offset, LE).unwrap() as f32
                }
            },
        _ => panic!("complex conversion called with real format parameter") // bug somewhere
    }
}
