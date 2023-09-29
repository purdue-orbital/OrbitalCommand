use bytes::{Bytes, Buf, BytesMut, BufMut};
use bytes::buf::Chain;

#[derive(Debug, Default)]
pub struct WtfECC {
	prev: u8
}

pub type BytesChain = Chain<Chain<Bytes, Bytes>, Bytes>;

impl WtfECC {
	pub const EXPANSION_RATIO: usize = 3;

	pub fn new() -> Self {
		Self::default()
	}

	pub fn reset(&mut self) {
		*self = Self::default();
	}

	pub fn encode_into(&mut self, data: &Bytes, b: &mut BytesMut, c: &mut BytesMut) {
		data.iter().for_each(|byte| {
			b.put_u8(!byte);
			c.put_u8(byte ^ self.prev);
			self.prev = *byte;
		});
	}

	pub fn encode(&mut self, data: Bytes) -> BytesChain {
		let mut b = BytesMut::with_capacity(data.remaining());
		let mut c = BytesMut::with_capacity(data.remaining());

		self.encode_into(&data, &mut b, &mut c);

		data.chain(b.freeze()).chain(c.freeze())
	}

	/// when decoding lots of data, you should use `self.encode`
	pub fn encode_to_bytes(&mut self, data: Bytes) -> Bytes {
		let mut output = self.encode(data);

		output.copy_to_bytes(output.remaining())
	}

	pub fn decode_into(&mut self, src: &mut Bytes, dst: &mut BytesMut) {
		let chunk_len = src.len() / Self::EXPANSION_RATIO;
		let a = src.split_to(chunk_len);
		let b = src.split_to(chunk_len);
		let c = src;

		a.iter()
			.zip(b.iter())
			.zip(c.iter())
			.for_each(|((byte_a, byte_b), byte_c)| {
				let according_to_b = !byte_b; // data according to b
				let according_to_c = byte_c ^ self.prev; // data according to c

				let bf_ab = byte_a ^ according_to_b; // bit flips between a & acc_b
				let bf_ac = byte_a ^ according_to_c; // bit flips between a & acc_c

				let bf_consensus = bf_ab & bf_ac; // agreed upon bit flips

				self.prev = byte_a ^ bf_consensus; // flip the bits

				dst.put_u8(self.prev)
			});
	}

	pub fn decode(&mut self, src: &mut Bytes) -> Bytes {
		let mut dst = BytesMut::with_capacity(src.len() / 3);

		self.decode_into(src, &mut dst);

		dst.freeze()
	}
}