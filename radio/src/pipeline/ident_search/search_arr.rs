use bitvec::prelude::*;
use bytes::{Buf, Bytes};
use rustdsp::ecc::wtf_ecc::WtfECC;

use super::super::frame::Frame;

#[derive(Debug)]
pub struct SearchArr {
	arr: [u8; Frame::ENCODED_IDENT_LENGTH],
	offset: u8,
	head: u8,
}

impl SearchArr {
	const LAST: usize = Frame::ENCODED_IDENT_LENGTH - 1;

	pub fn new() -> Self {
		Self {
			arr: [0; Frame::ENCODED_IDENT_LENGTH],
			offset: 0,
			/// left over bits
			head: 0,
		}
	}

	pub fn push(&mut self, x: u8) -> Option<ShiftInfo> {
		let mut bit;

		self.head = x;
		self.offset = 8;

		while self.offset > 0 {
			bit = self.head >> 7; // get the first bit

			// update state
			self.head <<= 1;
			self.offset -= 1;

			// move all the bits and insert the new first bit at the end
			let view = self.arr.view_bits_mut::<Msb0>();
			view.shift_left(1);
			self.arr[Self::LAST] |= bit;

			if self.is_match() {
				dbg!(&self.arr);
				dbg!(&self.offset);
				return Some(self.info());
			}
		}

		None
	}
	fn is_match(&self) -> bool {
		let mut decoder = WtfECC::new();
		let mut bytes = Bytes::from(self.arr.to_vec());
		let mut output = decoder.decode(&mut bytes);
		let mut x = [0; 4];
		output.copy_to_slice(&mut x);

		x == Frame::IDENT
	}

	fn info(&self) -> ShiftInfo {
		ShiftInfo {
			offset: self.offset,
			head: self.head,
		}
	}
}


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ShiftInfo { // TODO: replace with Shifter (would mean adjusting the tests)
	/// number of bits leftover
	pub offset: u8,
	/// the leftmost `self.offset` bits are what is leftover
	pub head: u8,
}

#[cfg(test)]
mod tests {
	use bytes::Buf;
	use super::*;

	fn encoded_ident() -> Bytes {
		let mut encoder = WtfECC::new();
		encoder.encode_to_bytes(Bytes::from_static(&Frame::IDENT))
	}

	#[test]
	fn test_is_match() {
		let mut encoded_ident = encoded_ident();

		// force searcher.arr to be the decoded bytes
		let mut searcher = SearchArr::new();
		encoded_ident.copy_to_slice(&mut searcher.arr);

		assert!(searcher.is_match());
	}

	#[test]
	fn test_push_1() {
		let mut encoded_ident = encoded_ident();
		let mut searcher = SearchArr::new();

		while encoded_ident.remaining() > 1 {
			searcher.push(encoded_ident.get_u8());
		}

		let ans = searcher.push(encoded_ident.get_u8());

		assert_eq!(ans, Some(ShiftInfo {
			offset: 0,
			head: 0,
		}));
	}

	#[test]
	fn test_push_2() {
		let mut bytestream = encoded_ident().to_vec();
		let mut searcher = SearchArr::new();
		let mut ans = None;

		// put a 0 byte at the start
		bytestream.insert(0, 0);

		for b in bytestream {
			ans = searcher.push(b);
		}

		assert_eq!(ans, Some(ShiftInfo {
			offset: 0,
			head: 0,
		}));
	}

	#[test]
	fn test_push_3() {
		let mut byte_stream = encoded_ident().to_vec();
		let mut searcher = SearchArr::new();
		let mut ans = None;

		// put a byte at start and end
		byte_stream.insert(0, 0xA3);
		byte_stream.push(0b10101010);

		let bit_thingy = byte_stream.view_bits_mut::<Msb0>();
		bit_thingy.shift_left(1);

		for b in byte_stream {
			ans = searcher.push(b);

			if ans.is_some() {
				break
			}
		}

		assert_eq!(ans, Some(ShiftInfo {
			offset: 1,
			head: 0b10000000,
		}));
	}

	#[test]
	fn test_data_recover_x() {
		for x in 0..8 {
			test_recover_data(
				&[240],
				&[0xFF, 0x42, 42, 0, 0b01010101],
				x
			)
		}
	}

	fn test_recover_data(pre: &[u8], post: &[u8], shift_left_by: usize) {
		let desired_data = post.to_vec();
		let mut enc_ident = encoded_ident().to_vec();
		let mut searcher = SearchArr::new();
		let mut ans = None;

		let mut byte_stream = Vec::with_capacity(
			pre.len()
			+ enc_ident.len()
			+ post.len()
		);

		byte_stream.append(&mut pre.to_vec());
		byte_stream.append(&mut enc_ident);
		byte_stream.append(&mut post.to_vec());

		let bit_thingy = byte_stream.view_bits_mut::<Msb0>();
		bit_thingy.shift_left(shift_left_by);

		let mut i = 0;
		for b in byte_stream.clone() {
			ans = searcher.push(b);
			i += 1;

			if ans.is_some() {
				break
			}
		}

		let mut data = byte_stream[i..].to_vec();

		assert!(ans.is_some());
		let offset = ans.unwrap().offset;
		assert_eq!(offset, shift_left_by as u8);

		// just to be extra sure that we don't accidentally peek at them
		drop(byte_stream);

		let mut space: u16 = (ans.unwrap().head as u16) << offset;

		for i in 0..data.len() {
			space |= data[i] as u16;
			space = space.rotate_right(offset as u32);
			data[i] = space as u8;

			space = space.swap_bytes();
			space &= 255;
			space <<= offset;
		}

		eprint_bin(&data, Some("data fixed (in theory)")); // DEBUG

		assert_eq!(desired_data, data);
	}

	fn eprint_bin(arr: &[u8], tag: Option<&str>) {
		if let Some(s) = tag {
			eprintln!("{}:", s);
		}
		for b in arr {
			eprintln!("{:#010b}, {b:#3}", b);
		}
		eprintln!();
	}
}
