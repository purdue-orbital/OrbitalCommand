mod decode;
mod encode;
mod common;

pub mod prelude {
	pub use super::decode::DecoderState;
	// pub use super::decode::RcDecoderState;
	pub use super::encode::EncoderState;
}

#[cfg(test)]
mod tests {
	use super::prelude::*;
	use crate::common::*;

	#[test]
	fn test_round_trip_1() {
		let bytes = vec![0xFF, 0x10, 0x00];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}

	#[test]
	fn test_round_trip_full_00() {
		let bytes = vec![0x00; 127];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}

	#[test]
	fn test_round_trip_full_ff() {
		let bytes = vec![0xFF; 127];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}

	#[test]
	fn test_round_trip_ff_ff_00_00() {
		let bytes = vec![0xFF, 0xFF, 0x00, 0x00];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}

	#[test]
	fn test_round_trip_ff_00_00_00_00() {
		let bytes = vec![0xFF, 0x00, 0x00, 0x00, 0x00];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}

	#[test]
	fn test_round_trip_all_3_bit_sequences() {
		let bytes = vec![
			0b11110000,
			0b11001100,
			0b10101010,
		];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		
		let output = decoder.read();

		assert_eq!(bytes, output);
	}

	#[test]
	fn test_round_trip_staircase() {
		let bytes = vec![
			0b10000000,
			0b11000000,
			0b11100000,
			0b11110000,
			0b11111000,
			0b11111100,
			0b11111110,
			0b11111111,
		];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}
	
	#[test]
	fn test_round_trip_inverted_staircase() {
		let bytes = vec![
			0b11111111,
			0b01111111,
			0b00111111,
			0b00011111,
			0b00001111,
			0b00000111,
			0b00000011,
			0b00000001,
		];

		let mut encoder: EncoderState<u8> = EncoderState::default();
		let data_encoded = encoder.push_slice(&bytes);

		let mut decoder = DecoderState::new(bytes.len());
		decoder.push_slice(&data_encoded);
		let output = decoder.read();

		assert_eq!(bytes, output);
	}
}
