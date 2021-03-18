use std;
use std::io::Read;
use byte::*;
use rustfft::num_complex::Complex;
mod dsp;

fn cs16_le_to_cf32(src: &[u8], dst: &mut [Complex<f32>]) {
    let mut offset = 0;
    for v in dst.iter_mut() {
        *v = Complex{
            re: src.read_with::<i16>(&mut offset, LE).unwrap() as f32,
            im: src.read_with::<i16>(&mut offset, LE).unwrap() as f32
        }
    }
}

fn main() -> std::io::Result<()> {
    let (mut dsp, bufsize) = dsp::DspState::init(1024, 1.0 / std::i16::MAX as f32);

    // buffer for raw input data
    let mut rawbuf: Vec<u8> = vec![0; 4 * bufsize.new];

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
        cs16_le_to_cf32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total]);

        dsp.process(&buf)?;
    }
    Ok(())
}
