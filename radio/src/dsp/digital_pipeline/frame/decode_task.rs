use std::thread;

use bytes::Bytes;
use flume::{Receiver, Sender};

use crate::SEND_EXPECT_MSG;
use crate::wtf_ecc::WtfECC;

/// decodes each `Bytes` struct using WtfECC and transmits the decoded bytes
#[derive(Debug)]
pub struct Task {
	rx: Receiver<Bytes>,
	tx: Sender<Bytes>,
	decoder: WtfECC,
}

impl Task {
	const NAME: &str = "bytes decode";

	/// setup the state for this task and build the thread
	pub fn new(rx: Receiver<Bytes>) -> (Self, Receiver<Bytes>) {
		let (output_tx, output_rx) = flume::unbounded(); // should this be bounded?

		(
			Self {
				rx,
				tx: output_tx,
				decoder: WtfECC::new(),
			},
			output_rx
		)
	}

	/// starts the thread for the task
	pub fn start(mut self) {
		thread::Builder::new().name(Self::NAME.to_string()).spawn(move || {
			while let Ok(mut bin) = self.rx.recv() {
				let data = self.decoder.decode(&mut bin);
				self.tx.send(data).expect(SEND_EXPECT_MSG);
				self.reset();
			}
		}).expect(Self::NAME);
	}

	fn reset(&mut self) {
		self.decoder = WtfECC::new();
	}
}
