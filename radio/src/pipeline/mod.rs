pub mod middle_man;
pub mod frame;
pub mod ident_search;

pub mod prelude {
	pub use super::frame::{encode_task, decode_task};
	pub use super::ident_search::search_task;
	pub use super::create_bytes_channel;
}

use flume::{Sender, Receiver};
use bytes::Bytes;

const SEND_EXPECT_MSG: &str = "Output RX shouldn't be dropped";

pub fn create_bytes_channel() -> (Sender<Bytes>, Receiver<Bytes>) {
	flume::unbounded()
}