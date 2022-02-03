use std;
use std::io::Read;
use rustfft::num_complex::Complex;
use zmq;

#[macro_use]
extern crate clap;

mod dsp;

mod inputformats;
use inputformats::*;


fn parse_configuration() -> (dsp::DspParams, InputFormat, String) {
    use clap::{App};
    let matches = App::new("spektri")
        .args_from_usage("
            -s, --fftsize=[SIZE]             'FFT size'
            -b, --fftbuf=[NUMBER]            'Number of FFTs in each input buffer (adjust to optimize performance)'
            -a, --averages=[NUMBER]          'Number of FFTs averaged for spectrum'
            -f, --inputformat=[FORMAT]       'Input signal format'
            -F, --spectrumformat=[FORMAT]    'Spectrum output format'
            -o, --filters=[PARAMETERS]...    'Filter parameters'
                --zmqbind=[ADDRESS]          'ZeroMQ binding address'
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
            value_t!(matches, "spectrumformat", dsp::SpectrumFormat)
            .unwrap_or(dsp::SpectrumFormat::U8),
        spectrum_averages:
            value_t!(matches, "averages", u32)
            .unwrap_or(2000),
        filters:
            values_t![matches, "filters", String]
            .unwrap_or_else(|_| Vec::new())
            .iter()
            .map(|x| parse_filter_params(x))
            .collect::<Vec<dsp::FilterParams>>(),
    },
    inputformat,
    value_t!(matches, "zmqbind", String).unwrap_or("ipc:///tmp/spektri.zmq".into())
    )
}


fn parse_filter_params(s: &str) -> dsp::FilterParams {
    use std::collections::HashMap;

    // , might be a nicer separator for parameters, but clap with
    // these settings doesn't seem to like arguments with commas,
    // so let's use ; instead.
    let m: HashMap<_, _> =
        s.split(";")
        .map(|x| {
            x.split_once('=').unwrap() // TODO: handle errors
        })
        .collect();

    dsp::FilterParams {
        // TODO: handle errors
        freq: m.get("freq").unwrap().parse().unwrap(),
        ifft_size: m.get("bins").unwrap().parse().unwrap(),
        output: dsp::output::OutputParams {
            filename: if let Some(v) = m.get("file")  { Some(v.to_string()) } else { None },
            topic:    if let Some(v) = m.get("topic") { Some(v.to_string()) } else { None },
        },
    }
}


fn main() -> std::io::Result<()> {
    let (dspparams, inputformat, zmqbind) = parse_configuration();

    let zctx = zmq::Context::new();
    let sock = zctx.socket(zmq::PUB).unwrap();
    // TODO: set SNDBUF and HWM sizes
    sock.bind(&zmqbind).unwrap();

    if is_input_format_complex(inputformat) {
        mainloop_complex(dspparams, inputformat, sock)
    } else {
        mainloop_real(   dspparams, inputformat, sock)
    }?;
    Ok(())
}


fn mainloop_complex(
    dspparams: dsp::DspParams,
    fmt: InputFormat,
    sock: zmq::Socket,
) -> std::io::Result<()> {
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
        let systemtime = std::time::SystemTime::now();
        convert_to_cf32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total], fmt);

        let metadata = dsp::Metadata {
            systemtime: systemtime,
        };

        dsp.process_complex(&buf, &metadata, &sock)?;
    }
    Ok(())
}

fn mainloop_real(
    dspparams: dsp::DspParams,
    fmt: InputFormat,
    sock: zmq::Socket,
) -> std::io::Result<()> {
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
        let systemtime = std::time::SystemTime::now();
        convert_to_f32(&rawbuf, &mut buf[bufsize.overlap .. bufsize.total], fmt);

        let metadata = dsp::Metadata {
            systemtime: systemtime,
        };

        dsp.process_real(&buf, &metadata, &sock)?;
    }
    Ok(())
}
