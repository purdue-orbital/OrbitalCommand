use std::thread;
use std::io;

use bytes::Bytes;

use radio::pipeline::prelude::*;
use rustdsp::{Demodulators, Modulators};

const SAMPLES_PER_BIT: usize = 3;
const SAMPLE_RATE: f32 = 1e6;

fn main() {
	let (tx_start, rx_start) = create_bytes_channel();
	let (encoder, rx_encoder) = encode_task::Task::new(rx_start);
	let (modulator, rx_mod) = modulate::Task::new(
		rx_encoder,
		Modulators::new(SAMPLES_PER_BIT, SAMPLE_RATE)
	);
	
	let (sample_search, rx_encoded_bytes) = sample_ident_search::Task::new(
		rx_mod,
		SAMPLES_PER_BIT,
		Demodulators::new(SAMPLES_PER_BIT, SAMPLE_RATE)
	);

	let (decoder, rx_bytes) = decode_task::Task::new(rx_encoded_bytes);

	decoder.start();
	sample_search.start();    
	modulator.start();
	encoder.start();

	
	thread::spawn(move || {
		loop {
			dbg!(String::from_utf8(rx_bytes.recv().unwrap().to_vec()).unwrap());
		}
	});
	
	loop {
		let mut input = String::new();
		io::stdin().read_line(&mut input).unwrap();

		tx_start.send(Bytes::from(input)).unwrap();
	}
}
