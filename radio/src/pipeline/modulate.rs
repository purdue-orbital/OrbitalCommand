use std::thread;

use bytes::{Bytes, Buf};
use flume::{Receiver, Sender};
use num_complex::Complex;

use rustdsp::Modulators;

use crate::pipeline::SEND_EXPECT_MSG;

pub struct Task {
	rx: Receiver<Bytes>,
	tx: Sender<Vec<Complex<f32>>>,
	modulator: Modulators,
}


impl Task {
	const NAME: &str = "modulation";
	const CHUNK_SIZE: usize = 50; // how many byes get modulated at a time
	
	pub fn new(rx: Receiver<Bytes>, modulator: Modulators) -> (Self, Receiver<Vec<Complex<f32>>>) {
		let (tx, out_rx) = flume::unbounded();
		
		(Self {
			rx,
			tx,
			modulator,
		}, out_rx)
	}

	pub fn start(self) {
		thread::Builder::new().name(Self::NAME.to_string()).spawn(move || {
			while let Ok(bin) = self.rx.recv() {
				for each in bin.chunks(Self::CHUNK_SIZE) {
					self.tx.send(
						self.modulator.bpsk(each)
					).expect(SEND_EXPECT_MSG);
				}
			}
		}).expect(Self::NAME);
	}
}
