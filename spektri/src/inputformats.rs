/*
 * Input formats and conversion functions
 */

use byte::*;
use rustfft::num_complex::Complex;


arg_enum! { // needed for command line parsing
    #[derive(Debug, Copy, Clone)]
    pub enum InputFormat {
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

pub fn bytes_per_input_sample(fmt: InputFormat) -> usize {
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

pub fn is_input_format_complex(fmt: InputFormat) -> bool {
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
pub fn input_format_scaling(fmt: InputFormat) -> f32 {
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


pub fn convert_to_f32(src: &[u8], dst: &mut [f32], fmt: InputFormat) {
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

pub fn convert_to_cf32(src: &[u8], dst: &mut [Complex<f32>], fmt: InputFormat) {
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
