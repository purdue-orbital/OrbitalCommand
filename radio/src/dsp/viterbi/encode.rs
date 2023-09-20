use std::ops::BitXor;
use crate::dsp::viterbi::common::combine;


#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// represents the internal state of multiple encoders. (each bit is its own encoder)
/// 
/// for more detail on how this works see [this video](https://youtu.be/kRIfpmiMCpU)
pub struct EncoderState<T: BitXor + Copy>(pub T, pub T);

impl<T: BitXor<Output = T> + Copy> EncoderState<T> {
	/// input a chunk to the encoder, updating state and returning the 2 chunks that should be transmitted
	pub fn push(&mut self, chunk: T) -> (T, T) {
		let ans = (
			self.1 ^ chunk,
			self.0 ^ self.1 ^ chunk
		);

		self.update(chunk);

		ans
	}

	#[inline]
	/// update the state.
	fn update(&mut self, chunk: T) {
		self.1 = self.0;
		self.0 = chunk;
	}
}

impl From<u8> for EncoderState<u8> {
	fn from(value: u8) -> Self {
		match value {
			0 => Self(0x00, 0x00),
			1 => Self(0xFF, 0x00),
			2 => Self(0x00, 0xFF),
			3 => Self(0xFF, 0xFF),
			_ => unreachable!()
		}
	}
}

impl From<EncoderState<u8>> for u8 {
	fn from(value: EncoderState<u8>) -> Self {
		combine(value.0, value.1)
	}
}

impl EncoderState<u8> {
	/// does the same thing as input, but it combines the 2 bytes into a bit pair
	/// 
	/// NOTE: this won't work in a usefull manner if you are using the EncoderState to encode multiple bits side by side
	/// its only purpose really is for testing
	pub fn push_return_bitpair(&mut self, byte: u8) -> u8 { // todo kill???
		let (s0, s1) = self.push(byte);
		combine(s0, s1)
	}

	pub fn push_slice(&mut self, arr: &[u8]) -> Vec<u8> {
		let mut ans = Vec::with_capacity(arr.len() * 2);

		for each in arr {
			let pair = self.push(*each);
			
			ans.push(pair.0);
			ans.push(pair.1);
		}

		ans
	}
}

