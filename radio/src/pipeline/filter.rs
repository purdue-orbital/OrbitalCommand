use std::thread;

use bytes::Bytes;
use flume::{Receiver, Sender};
use num_complex::Complex;
use rustdsp::filters::fir::{
	Windowing,
	shapes::WindowShapes
};

use crate::pipeline::SEND_EXPECT_MSG;

type VecComplexF32 = Vec<Complex<f32>>;

pub struct Task {
    rx: Receiver<VecComplexF32>,
    tx: Sender<VecComplexF32>,
	filter: Windowing,
}

impl Task {
    const NAME: &str = "filter";

	/// setup the state for this task and build the thread
	pub fn new(rx: Receiver<VecComplexF32>, filter: Windowing) -> (Self, Receiver<VecComplexF32>) {
		let (output_tx, output_rx) = flume::unbounded(); // should this be bounded?

		(
			Self {
				rx,
				tx: output_tx,
				filter,
			},
			output_rx
		)
	}

	/// starts the thread for the task
	pub fn start(mut self) {
		thread::Builder::new().name(Self::NAME.to_string()).spawn(move || {
			while let Ok(mut symbol) = self.rx.recv() {
				self.filter.run(&mut symbol);

				self.tx.send(symbol).expect(SEND_EXPECT_MSG);
			}
		}).expect(Self::NAME);
	}
}
