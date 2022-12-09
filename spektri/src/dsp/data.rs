//! This file defines various data types used in several parts of the code.
//! There are also functions to encode them for files or ZeroMQ messages.

use byte;
use byte::{BytesExt, LE};


/// Metadata for measurement records or buffers of samples
pub struct Metadata {
    /// Sequence number of processing block
    pub seq: u64,
    /// System time when processing block was received
    pub systemtime: std::time::SystemTime,
    // SDR timestamp could be added here as well but it's not implemented at the moment.
}


/// Information about FFT results
#[derive(Copy, Clone)]
pub struct FftInfo {
    /// Input sample rate
    pub fs:      f64,
    /// Input center frequency
    pub fc:      f64,
    /// FFT size
    pub size:    usize,
    /// Is the input signal real (false) or complex (true)
    pub complex: bool,
}


/// Information about signal data
pub struct SignalInfo {
    /// Sample rate
    pub fs: f64,
    /// Center frequency
    pub fc: f64,
}


/// Information about spectrum data
pub struct SpectrumInfo {
    /// Spacing of bins in frequency
    pub fd: f64,
    /// Frequency of the first bin
    pub f0: f64,
}

pub enum MessageType {
    Status   = 0x20,
    Waveform = 0x40,
    Spectrum = 0x60,
}

// Data format is encoded as:
// Highest 2 bits:
//   0 = real
//   1 = complex
//   2 = reserved
//   3 = reserved
// Next 2 bits:
//   0 = signed two's complement integer (or fixed point)
//   1 = float
//   2 = unsigned integer (or fixed point)
//   3 = reserved
// Next 3 bits: Number of bits per number
//   0 = reserved
//   1 = reserved
//   2 = 8
//   3 = 12
//   4 = 16
//   5 = 24
//   6 = 32
//   7 = 64
// Lowest 1 bit:
//   0 = little endian (or not applicable)
//   1 = big endian

arg_enum! { // needed for command line parsing
    #[derive(Debug, Copy, Clone)]
    pub enum DataFormat {
        S8     = 0x04, // real signed 8-bit
        S16le  = 0x08, // real signed 16-bit, little endian
        S16be  = 0x09, // real signed 16-bit, big endian
        F32le  = 0x1C, // real float 32-bit, little endian
        F32be  = 0x1D, // real float 32-bit, big endian
        U8     = 0x24, // real unsigned 8-bit

        Cs8    = 0x44, // complex signed 8-bit
        Cs16le = 0x48, // complex signed 16-bit, little endian
        Cs16be = 0x49, // complex signed 16-bit, big endian
        Cf32le = 0x5C, // complex float 32-bit, little endian
        Cf32be = 0x5D, // complex float 32-bit, big endian
        Cu8    = 0x64, // complex unsigned 8-bit
    }
}


const PROTOCOL_VERSION: u8 = 2;


/// Serialize metadata for a single measurement record.
/// The serialized metadata is placed in the beginning of each record
/// of signal or spectrum data.
///
/// Sequence number is taken as a separate parameter instead of
/// using metadata.seq because for some cases (spectrum data)
/// the sequence number of a measurement record is not the same
/// as the sequence number of a processing block.
pub fn serialize_metadata(
    buf: &mut [u8],
    offset: &mut usize,
    metadata: &Metadata,
    seq: u64, // Sequence number of samples
) -> byte::Result<()> {
    use std::time::UNIX_EPOCH;

    let (secs, nanosecs) = match metadata.systemtime.duration_since(UNIX_EPOCH) {
        Ok(d) => { (d.as_secs(), d.subsec_nanos()) },
        // If duration_since fails, let's just write zeros there.
        // Maybe we don't really need to handle it in any special way.
        Err(_) => { (0,0) },
    };

    buf.write_with(offset, seq, LE)?;
    buf.write_with(offset, secs, LE)?;
    buf.write_with(offset, nanosecs, LE)?;
    // Reserved
    buf.write_with(offset, 0u32, LE)?;

    Ok(())
}


/// Serialize topic for signal data.
/// The topic encodes sample rate and center frequency of the signal.
pub fn serialize_signal_topic(
    info:   &SignalInfo,
) -> [u8; 24] {
    let mut buf = [0u8; 24];

    buf[0] = PROTOCOL_VERSION;
    buf[1] = MessageType::Waveform as u8;
    buf[2] = DataFormat::Cf32le as u8;

    let mut offset = 8;
    // These should always fit into the buffer, so unwrap only panics
    // if there's a bug in this function.
    buf.write_with(&mut offset, info.fs, LE).unwrap();
    buf.write_with(&mut offset, info.fc, LE).unwrap();

    buf
}


/// Serialize topic for spectrum data.
pub fn serialize_spectrum_topic(
    info:   &SpectrumInfo,
) -> [u8; 24] {
    let mut buf = [0u8; 24];

    buf[0] = PROTOCOL_VERSION;
    buf[1] = MessageType::Spectrum as u8;
    buf[2] = DataFormat::U8 as u8;

    let mut offset = 8;
    buf.write_with(&mut offset, info.fd, LE).unwrap();
    buf.write_with(&mut offset, info.f0, LE).unwrap();

    buf
}
