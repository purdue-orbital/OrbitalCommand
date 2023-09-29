use std::thread;
use std::mem;

use bytes::{Bytes, Buf, BytesMut, BufMut};
use flume::{Receiver, Sender};
use crate::dsp::wtf_ecc::WtfECC;
use crate::pipeline::SEND_EXPECT_MSG;

use super::search_arr::*;


/// receives 8 bits at a time (in the form of a `u8`) and looks for `Frame::IDENT`
///
/// once it finds that, it shifts the bytes to capture the frame length
///
/// then it captures frame length times `EXPANSION_RATIO` bytes (shifted as needed) and sends them
/// onto the next step
///
/// NOTE: the output is not decoded!!!
#[derive(Debug)]
pub struct Task {
	rx: Receiver<u8>,
	tx: Sender<Bytes>,
	state: State,
	searcher: SearchArr,
	shifter: Shifter,
	data: BytesMut,
}

impl Task {
	const NAME: &str = "byte capture";
	const LEN_LEN: usize = 2 * WtfECC::EXPANSION_RATIO; // length of len (I know that's weird)

	/// setup the state for this task
	pub fn new(rx: Receiver<u8>) -> (Self, Receiver<Bytes>) {
		let (output_tx, output_rx) = flume::unbounded();

		(
			Self {
				rx,
				tx: output_tx,
				state: State::Ident,
				searcher: SearchArr::new(),
				shifter: Shifter::default(),
				data: Default::default(),
			},
			output_rx
		)
	}

	/// starts the thread for the task
	pub fn start(self) {
		thread::Builder::new()
			.name(Self::NAME.to_string())
			.spawn(move || self.execute())
			.expect(Self::NAME);
	}

	fn execute(mut self) {
		while let Ok(input) = self.rx.recv() {
			match self.state {
				State::Ident => {
					if let Some(shift_info) = self.searcher.push(input) {
						self.shifter = shift_info.into();
						self.state = State::Len;
					} // else, just continue on
				},

				State::Len => {
					if self.data.len() < Self::LEN_LEN { // still receiving the len
						self.push(input);
					} else { // len has been received
						let mut decoder = WtfECC::new();
						let mut output = decoder.decode(
							&mut self.data.copy_to_bytes(Self::LEN_LEN)
						);

						let len = output.get_u16_le() as usize;

						self.data = BytesMut::with_capacity(len * WtfECC::EXPANSION_RATIO);
						self.state = State::Data;
					}
				},

				State::Data => {
					// capture a byte
					self.push(input);

					// if we have captured all the bytes, send them to the next task
					if self.data.len() == self.data.capacity() {
						self.send_and_reset();
					}

				}
			}
		}
	}

	fn push(&mut self, input: u8) {
		self.shifter.push(input, &mut self.data);
	}

	fn send_and_reset(&mut self) {
		let captured_bytes = self.take_data(Self::LEN_LEN).freeze();
		self.tx.send(captured_bytes).expect(SEND_EXPECT_MSG);

		self.searcher = SearchArr::new();

		self.shifter = Shifter::default();
		self.state = State::Ident;
	}

	pub fn take_data(&mut self, capacity: usize) -> BytesMut {
		let mut new_bytes = BytesMut::with_capacity(capacity);
		mem::swap(&mut self.data, &mut new_bytes);

		new_bytes
	}
}

#[derive(Debug)]
enum State {
	Ident,
	Len,
	Data
}

#[derive(Debug, Default)]
struct Shifter {
	carry: u16,
	offset: u32,
}

impl From<ShiftInfo> for Shifter {
	fn from(value: ShiftInfo) -> Self {
		let offset = value.offset;

		Self {
			carry: (value.head as u16) << offset,
			offset: offset as u32,
		}
	}
}

impl Shifter {
	pub fn push(&mut self, input: u8, dst: &mut BytesMut) {
		self.carry |= input as u16;
		self.carry = self.carry.rotate_right(self.offset);

		dst.put_u8(self.carry as u8);

		self.carry &= 0x00FF;
		self.carry <<= self.offset;
	}
}