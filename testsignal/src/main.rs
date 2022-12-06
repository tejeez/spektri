//! Generate a frequency sweep across the spectrum and write it to stdout.

// This code could certainly be improved and cleaned up.
// It was one of the first Rust programs I've written.
// Support for multiple output formats was added as an afterthought
// and I did not really have time to reorganize the code.

use std::f32::consts::PI;
use std;
use std::io::Write;
use byte::*;

#[macro_use]
extern crate clap;

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    enum DataFormat {
        S16le  = 0x08, // real signed 16-bit, little endian
        F32le  = 0x1C, // real float 32-bit, little endian

        Cs16le = 0x48, // complex signed 16-bit, little endian
        Cf32le = 0x5C, // complex float 32-bit, little endian
    }
}

fn main() {
    let matches = clap::App::new("testsignal")
        .args_from_usage("
            -f, --format=[FORMAT]            'Data format (s16le, f32le, cs16le or cf32le)'
            -n, --samples=[NUMBER]           'Number of samples to produce'
            ")
        .get_matches();

    let format = value_t!(matches, "format", DataFormat).unwrap();
    let is_iq = (format as u8 & 0xC0) == 0x40;
    let is_s16le = (format as u8 & 0x1F) == 0x08; // otherwise f32le
    let nsamples: i64 = value_t!(matches, "samples", i64).unwrap();

    let buflen: usize = 1024;
    let nbufs: i64 = nsamples / buflen as i64;

    let init_freq: f32 = -PI;
    let freq_step: f32 = 2.0*PI / (buflen as f32 * nbufs as f32);
    let mut phase: f32 = 0.0;

    let mut buf: Vec<u8> = vec![0; buflen * (if is_iq {2} else {1} ) * (if is_s16le {2} else {4})];

    let mut output = std::io::stdout();

    // Use an integer counter to avoid accumulating rounding errors
    // instead of something like freq += freq_step; on each iteration.
    let mut sample_counter: i64 = 0;
    for _j in 0..nbufs {
        let mut buf_offset: usize = 0;
        for _i in 0..buflen {
            let sample_i = phase.cos();
            if is_s16le {
                buf.write_with(&mut buf_offset, (sample_i * 32767.0) as i16, LE).unwrap();
            } else {
                buf.write_with(&mut buf_offset, sample_i, LE).unwrap();
            }
            if is_iq {
                let sample_q = phase.sin();
                if is_s16le {
                    buf.write_with(&mut buf_offset, (sample_q * 32767.0) as i16, LE).unwrap();
                } else {
                    buf.write_with(&mut buf_offset, sample_q, LE).unwrap();
                }
            }
            let freq = init_freq + freq_step * sample_counter as f32;
            phase = (phase + freq) % (2.0*PI);
            sample_counter += 1;
        }
        output.write_all(&buf).unwrap();
    }
}
