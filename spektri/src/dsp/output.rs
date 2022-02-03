// Write results into a file and/or publish them by ZeroMQ.

/*
 * Output serialization is separated from I/O, so that serialization can be
 * done in parallelized parts of the code while keeping I/O sequential.
 * This is useful since ZeroMQ sockets are not thread safe and it might also
 * be useful to keep a consistent ordering of output messages.
 */

use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use byte;
//use zmq;
use super::Metadata;


pub struct OutputParams {
	pub filename: Option<String>,
	pub topic:    Option<String>, // ZeroMQ publishing topic
}

pub struct Output {
	file:   Option<File>,
	topic:  Option<(Vec<u8>, Vec<u8>)>,
}

impl Output {
	pub fn init(
		params: &OutputParams,
	) -> Self {
		Self {
			file: if let Some(filename) = &params.filename {
				match File::create(filename) {
					Ok(file) => { Some(file) },
					Err(err) => {
						eprintln!("Could not create file: {}", err);
						None
					}
				}
			} else {
				// No file output
				None
			},
			topic: if let Some(topic) = &params.topic {
				Some((
					("d".to_owned() + &topic).into_bytes(), // data
					("m".to_owned() + &topic).into_bytes(), // metadata
				))
			} else {
				None
			},
		}
	}

	pub fn write(
		&mut self,
		buf: &[u8],
		sock: &zmq::Socket,
	) -> Result<(), Box<dyn Error>> {
		if let Some(topic) = &self.topic {
			sock.send(&topic.0, zmq::SNDMORE);
			sock.send(&buf, 0);
		}

		if let Some(file) = &mut self.file {
			file.write_all(&buf)?;
		}

		Ok(())
	}
}


pub fn serialize_metadata(
	buf: &mut [u8],
	offset: &mut usize,
	metadata: &Metadata,
	seq: u64, // Sequence number of samples
) -> Result<(), byte::Error> {
	use byte::*;
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
