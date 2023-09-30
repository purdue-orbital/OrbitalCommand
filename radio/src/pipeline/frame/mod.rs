use bytes::{Bytes, Buf};
use rustdsp::ecc::wtf_ecc::WtfECC;

pub mod encode_task;
pub mod decode_task;

#[derive(Debug)]
pub struct Frame {
	len: u16,
	bin: Bytes,
}

impl Frame {
	// NOTE: using big endian because it matches how its written
	pub const IDENT: [u8; 4] = 0b11110000111100001111000011110001_u32.to_be_bytes();
	pub const ENCODED_IDENT_LENGTH: usize = Self::IDENT.len() * WtfECC::EXPANSION_RATIO;

	/// creates a new `Frame`, returning `Err(bin.copy_to_bytes())` if `bin` is too big.
	pub fn new_from_bin(bin: impl Buf) -> Result<Self, Bytes> {
		let mut ans = Self::empty();
		ans.change_bin(bin)?;

		Ok(ans)
	}

	pub fn empty() -> Self {
		Self {
			len: 0,
			bin: Bytes::new()
		}
	}

	/// changes bin, returning `Err(new_bin.copy_to_bytes())` if `new_bin` is too big.
	pub fn change_bin(&mut self, mut new_bin: impl Buf) -> Result<(), Bytes> {
		let len = new_bin.remaining();

		// make sure that we don't try to transmit a message longer than the max that len allows for
		if len > (u16::MAX as usize) {
			Err(new_bin.copy_to_bytes(len)) // TODO: figure out how to return the buffer w/o converting to Bytes
		} else {
			self.len = len as u16;
			self.bin = new_bin.copy_to_bytes(len);

			Ok(())
		}
	}

	pub fn len(&self) -> u16 {
		self.len
	}

	pub fn len_usize(&self) -> usize {
		self.len as usize
	}

	pub fn encode(self) -> Bytes { // figure out how to return a `Buf`
		let mut encoder = WtfECC::new();
		let encoded_ident = encoder.encode_to_bytes(Bytes::from_static(&Self::IDENT));
		
		encoder.reset();
		let encoded_len = encoder.encode_to_bytes(Bytes::from(self.len.to_le_bytes().to_vec()));
		
		encoder.reset();
		let encoded_bin = encoder.encode(self.bin);

		let mut data = encoded_ident.chain(encoded_len).chain(encoded_bin);

		data.copy_to_bytes(data.remaining())
	}
}