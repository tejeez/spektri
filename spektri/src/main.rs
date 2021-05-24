use std;
use std::io::Read;
use byte::*;
use rustfft::num_complex::Complex;
//use rayon::prelude::*;

#[macro_use]
extern crate clap;

mod dsp;
mod multifft;

fn s16_le_to_f32(src: &[u8], dst: &mut [f32]) {
    let mut offset = 0;
    for v in dst.iter_mut() {
        *v = src.read_with::<i16>(&mut offset, LE).unwrap() as f32;
    }
}

fn cs16_le_to_cf32(src: &[u8], dst: &mut [Complex<f32>]) {
    let mut offset = 0;
    for v in dst.iter_mut() {
        *v = Complex{
            re: src.read_with::<i16>(&mut offset, LE).unwrap() as f32,
            im: src.read_with::<i16>(&mut offset, LE).unwrap() as f32
        }
    }
}

/*fn cs16_le_to_cf32_par(src: &[u8], dst: &mut [Complex<f32>]) {
    dst.par_iter_mut().enumerate().for_each( |(i,v)| {
        let mut offset = i * 4;
        *v = Complex{
            re: src.read_with::<i16>(&mut offset, LE).unwrap() as f32,
            im: src.read_with::<i16>(&mut offset, LE).unwrap() as f32
        }
    });
}*/

arg_enum! {
    #[derive(Debug)]
    enum InputFormat {
        S16le,   // real signed 16-bit, little endian
        Cs16le,  // complex signed 16-bit, little endian
    }
}

fn parse_configuration() -> (dsp::DspParams, InputFormat) {
    use clap::{Arg, App/*, SubCommand*/};
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
        complex: match inputformat { InputFormat::Cs16le => true, _ => false },
        fft_size: value_t!(matches, "fftsize", usize).unwrap_or(16384),
        scaling: 1.0 / std::i16::MAX as f32,
        ffts_per_buf: 8,
        spectrum_format: if matches.value_of("spectrumformat").unwrap_or("u8") == "u16" { dsp::SpectrumFormat::U16 } else { dsp::SpectrumFormat::U8 },
        spectrum_averages: value_t!(matches, "averages", u32).unwrap_or(2000),
    }, inputformat)
}


fn main() -> std::io::Result<()> {
    let (dspparams, inputformat) = parse_configuration();

    let (mut dsp, bufsize) = dsp::DspState::init(dspparams);

    let mut input = std::io::stdin();

    match inputformat {
    InputFormat::Cs16le => {
        // buffer for raw input data
        let mut rawbuf: Vec<u8> = vec![0; 4 * bufsize.new];

        // buffer for type converted data with overlap
        let mut buf: Vec<Complex<f32>> = vec![Complex{re:0.0,im:0.0}; bufsize.total ];

        'mainloop: loop {
            // copy the overlapping part to beginning of the buffer
            buf.copy_within(bufsize.new .. bufsize.total, 0);

            // read input samples, type convert and write to the rest of the buffer
            match input.read_exact(&mut rawbuf) {
                Err (_) => { break 'mainloop; }
                Ok  (_) => { }
            }
            cs16_le_to_cf32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total]);

            dsp.process_complex(&buf)?;
        }
    }
    _ => {
        // buffer for raw input data
        let mut rawbuf: Vec<u8> = vec![0; 2 * bufsize.new];

        // buffer for type converted data with overlap
        let mut buf: Vec<f32> = vec![0.0; bufsize.total ];

        'mainloop2: loop {
            // copy the overlapping part to beginning of the buffer
            buf.copy_within(bufsize.new .. bufsize.total, 0);

            // read input samples, type convert and write to the rest of the buffer
            match input.read_exact(&mut rawbuf) {
                Err (_) => { break 'mainloop2; }
                Ok  (_) => { }
            }
            s16_le_to_f32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total]);

            dsp.process_real(&buf)?;
        }
    }
    }
    Ok(())
}
