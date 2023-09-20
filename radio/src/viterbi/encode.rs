use std::ops::BitXor;

use crate::common::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// represents the internal state of multiple encoders. (each bit is its own encoder)
/// 
/// for more detail on how this works see [this video](https://youtu.be/kRIfpmiMCpU)
pub struct EncoderState<T: BitXor + Copy>(T, T);

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

#[cfg(test)]
mod tests {
	use super::*;

	fn state_eq(state: &EncoderState<u8>, correct: u8) {
		let x: EncoderState<u8> = correct.into();
		assert_eq!(state, &x);
	}

	#[test]
	fn test_state_updating() {
		let mut state = EncoderState::<u8>::default();

		state.push(0x00);
		state_eq(&state, 0);

		state.push(0xFF);
		state_eq(&state, 1);

		state.push(0xFF);
		state_eq(&state, 3);
		state = 1.into();

		state.push(0x00);
		state_eq(&state, 2);

		state.push(0x00);
		state_eq(&state, 0);
		state = 2.into();

		state.push(0xFF);
		state_eq(&state, 1);
		state = 3.into();

		state.push(0x00);
		state_eq(&state, 2);
		state = 3.into();

		state.push(0xFF);
		state_eq(&state, 3);
	}

	#[test]
	fn test_encoder_ouptut() {
		let mut state: EncoderState<u8> = EncoderState::default();

		let pair = state.push(0xFF);
		assert_eq!(pair, (0xFF, 0xFF));
	}

	#[test]
	fn test_to_from_encoder_state() {
		for x in 0u8..4 {
			let state: EncoderState<u8> = x.into();
			assert_eq!(x, state.into());
		}
	}

	#[test]
	fn test_from_u8() {
		let arr_a: [EncoderState<u8>; 4] = [
			0.into(),
			1.into(),
			2.into(),
			3.into(),
		];

		let arr_b: [EncoderState<u8>; 4] = [
			EncoderState(0, 0),
			EncoderState(0xFF, 0),
			EncoderState(0, 0xFF),
			EncoderState(0xFF, 0xFF)
		];

		assert_eq!(arr_a, arr_b);
	}

	#[test]
	fn test_to_u8() {
		let arr_a: [u8; 4] = [
			EncoderState(0, 0).into(),
			EncoderState(0xFF, 0).into(),
			EncoderState(0, 0xFF).into(),
			EncoderState(0xFF, 0xFF).into(),
		];
			
		let arr_b: [u8; 4] = [0, 1, 2, 3];

		assert_eq!(arr_a, arr_b);
	}
}
