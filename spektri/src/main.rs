use std;
use std::io::Read;
use byte::*;
use rustfft::num_complex::Complex;
//use rayon::prelude::*;
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Arguments: (real|complex) FFT_SIZE");
        return Ok(())
    }

    let is_complex: bool = args[1] == "complex";

    let (mut dsp, bufsize) = dsp::DspState::init(dsp::DspParams {
        complex: is_complex,
        fft_size: args[2].parse().unwrap(),
        scaling: 1.0 / std::i16::MAX as f32,
        ffts_per_buf: 8,
        spectrum_format: dsp::SpectrumFormat::U8,
        spectrum_averages: 2000,
    });

    // buffer for raw input data
    let mut rawbuf: Vec<u8> = vec![0; {if is_complex {4} else {2}} * bufsize.new];

    let mut input = std::io::stdin();

    if is_complex {
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
    } else {
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
    Ok(())
}
