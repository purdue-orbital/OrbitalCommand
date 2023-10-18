use std::thread;

use bytes::Bytes;
use flume::{Receiver, Sender};

use crate::pipeline::SEND_EXPECT_MSG;

/// acts as a middle man for testing purposes
#[derive(Debug)]
pub struct Task {
    rx: Receiver<Bytes>,
    tx: Sender<u8>,
    thread: thread::Builder,
}

impl Task {
    const NAME: &str = "middle man";

    /// setup the state for this task and build the thread
    pub fn new(rx: Receiver<Bytes>) -> (Self, Receiver<u8>) {
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
            while let Ok(bin) = self.rx.recv() {
                for b in bin {
                    self.tx.send(b).expect(SEND_EXPECT_MSG);
                }
            }
        }).expect(Self::NAME);
    }
}
