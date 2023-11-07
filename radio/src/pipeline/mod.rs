pub mod filter;
pub mod frame;
pub mod ident_search;
pub mod middle_man;
pub mod sample_ident_search;

pub mod prelude {
    pub use super::create_bytes_channel;
    pub use super::filter;
    pub use super::frame::{decode_task, encode_task};
    pub use super::ident_search::search_task;
}

use bytes::Bytes;
use flume::{Receiver, Sender};

const SEND_EXPECT_MSG: &str = "Output RX shouldn't be dropped";

pub fn create_bytes_channel() -> (Sender<Bytes>, Receiver<Bytes>) {
    flume::unbounded()
}
