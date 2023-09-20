mod single_bit_decode;

use single_bit_decode::BitDecoderState;
use crate::dsp::viterbi::common::*;

use rayon::prelude::*;

#[derive(Debug)]
pub struct DecoderState {
	pub decoders: [BitDecoderState; 8]
}

impl DecoderState {
	pub fn new(len: usize) -> Self {
		Self {
			decoders: [
				BitDecoderState::new(len),
				BitDecoderState::new(len),
				BitDecoderState::new(len),
				BitDecoderState::new(len),
				BitDecoderState::new(len),
				BitDecoderState::new(len),
				BitDecoderState::new(len),
				BitDecoderState::new(len)
			]
		}
	}

	pub fn push(&mut self, byte0: u8, byte1: u8) {
		for i in 0..8 {
			self.decoders[i].push(byte0 & BIT_MASK[i], byte1 & BIT_MASK[i])
		}
	}

	pub fn push_slice(&mut self, arr: &[u8]) {
		let mut i = 0;

		while i < arr.len() {
			let byte0 = arr[i];

			i += 1;
			let byte1 = arr[i];

			self.push(byte0, byte1);

			i += 1;
		}
	}

	pub fn push_slice_para(&mut self, arr: &[u8]) {
		self.decoders.par_iter_mut()
			.enumerate()
			.for_each(|(b, decoder)| {
				let mut i = 0;

				while i < arr.len() {
					let bit0 = arr[i] & BIT_MASK[b];

					i += 1;
					let bit1 = arr[i] & BIT_MASK[b];

					decoder.push(bit0, bit1);

					i += 1;
				}
			})
	}

	pub fn read(mut self) -> Vec<u8> {
		let mut ans = self.decoders[0].read(BIT_MASK[0]);

		for x in 1..8 {
			let new = self.decoders[x].read(BIT_MASK[x]);

			debug_assert_eq!(ans.len(), new.len());

			for i in 0..ans.len() {
				ans[i] |= new[i];
			}
		}

		ans
	}
}