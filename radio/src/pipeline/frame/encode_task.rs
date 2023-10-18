use std::thread;

use bytes::{Buf, Bytes};
use flume::{Receiver, Sender};

use crate::pipeline::SEND_EXPECT_MSG;

use super::Frame;

/// uses each `Bytes` struct that it receives as a payload for a `Frame`
/// the `Frame` is then encoded using ECC and then sent on the channel that it returns upon creation
#[derive(Debug)]
pub struct Task {
    rx: Receiver<Bytes>,
    tx: Sender<Bytes>,
    thread: thread::Builder,
}

impl Task {
    const NAME: &str = "frame encode";

    /// setup the state for this task and build the thread
    pub fn new(rx: Receiver<Bytes>) -> (Self, Receiver<Bytes>) {
        let (output_tx, output_rx) = flume::unbounded(); // should this be bounded?

        (
            Self {
                rx,
                tx: output_tx,
                thread: thread::Builder::new().name(Self::NAME.to_string()),
            },
            output_rx
        )
    }

    /// starts the thread for the task
    pub fn start(self) {
        self.thread.spawn(move || {
            while let Ok(payload) = self.rx.recv() {
                if let Ok(frame) = Frame::new_from_bin(payload) {
                    let mut bin = frame.encode().chain(Bytes::from_static(&[0; 2]));
                    self.tx.send(
                        bin.copy_to_bytes(bin.remaining())
                    ).expect(SEND_EXPECT_MSG)
                } else {
                    // don't attempt to send this payload
                    eprintln!("Payload is too large to fit into Frame!!! (from {})", Self::NAME);
                }
            }
        }).expect(Self::NAME);
    }
}
