/* Program that generates a frequency sweep (chirp) across the spectrum
 * and writes it to stdout in 16-bit signed little-endian format. */

use std::f32::consts::PI;
use std;
use std::io::Write;
use byte::*;

fn main() {
    let is_iq: bool = false; // false for real signal, true for complex I/Q
    let buflen = 1024;
    let nbufs = 10000;

    let amplitude: f32 = 32000.0;
    let mut freq:  f32 = -PI;
    let freq_step: f32 = 2.0*PI / (buflen as f32 * nbufs as f32);
    let mut phase: f32 = 0.0;

    let mut buf: Vec<u8> = vec![0; if is_iq { 4*buflen } else { 2*buflen }];

    let mut output = std::io::stdout();

    for _j in 0..nbufs {
        let mut buf_offset: usize = 0;
        for _i in 0..buflen {
            let sample_i = (phase.cos() * amplitude) as i16;
            buf.write_with(&mut buf_offset, sample_i, LE).unwrap();
            if is_iq {
                let sample_q = (phase.sin() * amplitude) as i16;
                buf.write_with(&mut buf_offset, sample_q, LE).unwrap();
            }
            phase = (phase + freq) % (2.0*PI);
            freq += freq_step;
        }
        output.write_all(&buf).unwrap();
    }
}
