use std::collections::VecDeque;
use std::thread;

use bytes::{Bytes, Buf};
use flume::{Receiver, Sender};
use num_complex::Complex;
use rustdsp::Demodulators;

use crate::pipeline::frame::Frame;
use crate::pipeline::SEND_EXPECT_MSG;
use rustdsp::ecc::wtf_ecc::WtfECC;

mod stuff;

use stuff::*;

type VecComplexF32 = Vec<Complex<f32>>;

pub struct Task {
	rx: Receiver<VecComplexF32>,
	tx: Sender<Bytes>,
	demoder: Demodulators,
	samples_per_bit: usize,
	state: State,
	ident_ring_buffer: VecDeque<Complex<f32>>,
	sample_arr: VecDeque<Complex<f32>>,
	len: usize
}

impl Task {
	const NAME: &str = "samplewise ident search";
	const LEN_LEN: usize = 2 * WtfECC::EXPANSION_RATIO; // length of len (I know that's weird)

	fn samples_per_ident(&self) -> usize {
		self.samples_per_bit * 8 * Frame::ENCODED_IDENT_LENGTH
	}

	fn reset(&mut self) {
		self.ident_ring_buffer = vec![
			Complex::new(0., 0.);
			self.samples_per_ident()
		].into();

		self.state = State::Ident;
		self.sample_arr.clear();
		self.len = 0;
	}

	/// setup the state for this task and build the thread
	pub fn new(
		rx: Receiver<VecComplexF32>,
		samples_per_bit: usize,
		demoder: Demodulators,
	) -> (Self, Receiver<Bytes>) {
		let (output_tx, output_rx) = flume::unbounded();

		(
			Self {
				rx,
				tx: output_tx,
				demoder,
				samples_per_bit,
				state: State::Ident,
				ident_ring_buffer: VecDeque::with_capacity(samples_per_bit * 8 * Frame::ENCODED_IDENT_LENGTH),
				sample_arr: VecDeque::with_capacity(u16::MAX as usize * WtfECC::EXPANSION_RATIO),
				len: 0
			},
			output_rx,
		)
	}

	fn ring_shift_insert(&mut self, sample: Complex<f32>) {
		if self.ident_ring_buffer.len() >= self.samples_per_ident() {
			self.ident_ring_buffer.pop_back();
		}
		self.ident_ring_buffer.push_front(sample);
	}

	/// starts the thread for the task
	pub fn start(mut self) {
		thread::Builder::new()
			.name(Self::NAME.to_string())
			.spawn(move || {
				while let Ok(sample_packet) = self.rx.recv() {
					self.sample_arr.append(&mut sample_packet.into());

					match self.state {
						State::Ident => {
							// loops sample by sample
							while !self.sample_arr.is_empty() {
								let temp = self.sample_arr.pop_front().unwrap();
								self.ring_shift_insert(temp);

								let maybe_ident = self.demoder.bpsk(self.ident_ring_buffer.make_contiguous());
								// dbg!(&maybe_ident.len());

								if (maybe_ident.len() == 12 && maybe_ident[0] == 240) {
									dbg!(&maybe_ident);
								}

								let mut decoder = WtfECC::new();
								let mut plzbethefuckingident = decoder.decode(&mut Bytes::from(maybe_ident)).to_vec(); // maybe this is backwards?
								plzbethefuckingident.reverse();

								
								if plzbethefuckingident == Frame::IDENT {
									dbg!("IDENT FOUND!");
									// HELL YEAH!!!!

									self.state = State::Len;
									break;
								}
							}

							dbg!("no ident");
						},
						State::Len => {
							let num_samples = self.samples_per_bit * 8 * Self::LEN_LEN;

							if self.sample_arr.len() >= num_samples {
								let mut arr = Vec::with_capacity(num_samples);

								for _ in 0..num_samples {
									arr.push(self.sample_arr.pop_front().unwrap());
								}

								arr.reverse();
								let encoded_len = self.demoder.bpsk(&arr); // maybe this is backwards???
								let mut decoder = WtfECC::new();

								self.len = decoder.decode(&mut Bytes::from(encoded_len)).get_u16_le() as usize;
								self.state = State::Data;

								dbg!(self.len);
							}
						},
						State::Data => {
							let num_samples = self.samples_per_bit * 8 * WtfECC::EXPANSION_RATIO * self.len;

							dbg!(num_samples - self.sample_arr.len());

							if self.sample_arr.len() >= num_samples {
								let arr = self.sample_arr.make_contiguous();
								arr.reverse();
								let data = Bytes::from(self.demoder.bpsk(&arr)); // maybe this is backwards???
								
								self.tx.send(data).expect(SEND_EXPECT_MSG);
								self.reset();
							}
						},
					}
				}
			}).expect(Self::NAME);
	}
}
