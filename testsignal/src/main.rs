/* Program that generates a frequency sweep (chirp) across the spectrum
 * and writes it to stdout in 16-bit signed little-endian format. */

use std::f32::consts::PI;
use std;
use std::io::Write;
use byte::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Arguments: (real|complex) NUMBER_OF_SAMPLES");
        return
    }

    let is_iq: bool = args[1] == "complex";
    let nsamples: i64 = args[2].parse().unwrap();
    let buflen: usize = 1024;
    let nbufs: i64 = nsamples / buflen as i64;

    let amplitude: f32 = 32000.0;
    let init_freq: f32 = -PI;
    let freq_step: f32 = 2.0*PI / (buflen as f32 * nbufs as f32);
    let mut phase: f32 = 0.0;

    let mut buf: Vec<u8> = vec![0; if is_iq { 4*buflen } else { 2*buflen }];

    let mut output = std::io::stdout();

    /* Use an integer counter to avoid accumulating rounding errors
     * instead of something like freq += freq_step; on each iteration. */
    let mut sample_counter: i64 = 0;
    for _j in 0..nbufs {
        let mut buf_offset: usize = 0;
        for _i in 0..buflen {
            let sample_i = (phase.cos() * amplitude) as i16;
            buf.write_with(&mut buf_offset, sample_i, LE).unwrap();
            if is_iq {
                let sample_q = (phase.sin() * amplitude) as i16;
                buf.write_with(&mut buf_offset, sample_q, LE).unwrap();
            }
            let freq = init_freq + freq_step * sample_counter as f32;
            phase = (phase + freq) % (2.0*PI);
            sample_counter += 1;
        }
        output.write_all(&buf).unwrap();
    }
}
