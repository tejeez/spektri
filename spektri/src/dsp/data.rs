/// This file defines various data types used in several parts of the code.
/// There are also functions to encode them for files or ZeroMQ messages.

use byte;
use byte::{BytesExt, LE};


/// Metadata for measurement records or buffers of samples
pub struct Metadata {
    pub systemtime: std::time::SystemTime,
}


/// Information about signal data
pub struct SignalInfo {
	pub fs: f64, // Sample rate
	pub fc: f64, // Center frequency
}


/// Information about spectrum data
pub struct SpectrumInfo {
	pub fd: f64, // Spacing of bins in frequency
	pub f0: f64, // Frequency of the first bin
}


/// Serialize metadata for a single measurement record.
/// The serialized metadata is placed in the beginning of each record
/// of signal or spectrum data.
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
	buf[0] = 2;    // format version
	buf[1] = 0x50; // complex signal, CF32
	let mut offset = 8;
	buf.write_with(&mut offset, info.fs, LE);
	buf.write_with(&mut offset, info.fc, LE);
	buf
}


/// Serialize topic for spectrum data.
pub fn serialize_spectrum_topic(
	info:   &SpectrumInfo,
) -> [u8; 24] {
	let mut buf = [0u8; 24];
	buf[0] = 2;    // format version
	buf[1] = 0x61; // spectrum, U8
	let mut offset = 8;
	buf.write_with(&mut offset, info.fd, LE);
	buf.write_with(&mut offset, info.f0, LE);
	buf
}
