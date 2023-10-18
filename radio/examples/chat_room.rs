use std::io;
use std::thread;

use bytes::Bytes;

use radio::pipeline::middle_man;
use radio::pipeline::prelude::*;

// TODO: remove

fn main() {
    // setup the tasks to send
    let rx_input = user_input_thread();
    let (encoder, rx_encode) = encode_task::Task::new(rx_input);

    // TODO: replace w/ the radio sending and receiving task. REMEMBER TO START THEM!!!
    let (middle_man, rx_middle_man) = middle_man::Task::new(rx_encode);

    // setup the tasks to receive
    let (searcher, rx_search) = search_task::Task::new(rx_middle_man);
    let (decoder, rx_decode) = decode_task::Task::new(rx_search);
    output_thread(rx_decode);

    // start the tasks
    searcher.start();
    decoder.start();
    middle_man.start();
    encoder.start();
}

fn user_input_thread() -> flume::Receiver<Bytes> {
    let (tx, rx) = flume::bounded(1);

    thread::spawn(move || {
        let mut input = String::new();

        loop {
            io::stdin().read_line(&mut input).expect("failed to readline");

            tx.send(input.clone().into()).expect("failed to send");
        }
    });

    rx
}

fn output_thread(rx: flume::Receiver<Bytes>) {
    thread::spawn(move || {
        loop {
            while let Ok(bin) = rx.recv() {
                dbg!(String::from_utf8_lossy(&bin));
            }
        }
    });
}