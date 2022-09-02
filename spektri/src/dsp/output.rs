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
//use zmq;


pub struct OutputParams {
	pub filename: Option<String>,
}

pub struct Output {
	file:   Option<File>,
	topic:  Vec<u8>,
}

impl Output {
	pub fn init(
		params: &OutputParams,
		topic: &[u8],
	) -> Self {
		Self {
			file: if let Some(filename) = &params.filename {
				match File::create(filename) {
					Ok(file) => {
						// TODO: write header containing the topic
						// to beginning of the file
						Some(file)
					},
					Err(err) => {
						eprintln!("Could not create file: {}", err);
						None
					}
				}
			} else {
				// No file output
				None
			},
			topic: topic.to_vec(),
		}
	}

	pub fn write(
		&mut self,
		buf: &[u8],
		sock: &zmq::Socket,
	) -> Result<(), Box<dyn Error>> {
		sock.send(&self.topic, zmq::SNDMORE)?;
		sock.send(&buf, 0)?;

		if let Some(file) = &mut self.file {
			file.write_all(&buf)?;
		}

		Ok(())
	}
}
