use bytes::{Bytes, Buf, BytesMut, BufMut};

#[derive(Debug, Default)]
pub struct WtfECC {
	prev: u8
}

impl WtfECC {
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

	pub fn encode(&mut self, data: Bytes) -> bytes::buf::Chain<bytes::buf::Chain<bytes::Bytes, bytes::Bytes>, bytes::Bytes> {
		let mut b = BytesMut::with_capacity(data.remaining());
		let mut c = BytesMut::with_capacity(data.remaining());

		self.encode_into(&data, &mut b, &mut c);

		data.chain(b.freeze()).chain(c.freeze())
	}

	pub fn decode_into(&mut self, src: &mut Bytes, dst: &mut BytesMut) {
		let third = src.len() / 3;
		let a = src.split_to(third);
		let b = src.split_to(third);
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