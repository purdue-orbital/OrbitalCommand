const NUM_BYTES: usize = 10_000_000;
const SIZE_FRAME: usize = 50_000;
const NUM_FRAMES: usize = NUM_BYTES / SIZE_FRAME;
const ACTUAL_BYTES: usize = NUM_FRAMES * SIZE_FRAME;

use radio::pipeline::prelude::*;
use radio::pipeline::middle_man;
use rand::prelude::*;
use bytes::Bytes;

use std::thread;
use std::time;

fn main() {
	// setup the tasks of the pipeline
	let (tx_start, rx_start) = create_bytes_channel();
	let (encoder, rx_encoder) = encode_task::Task::new(rx_start);
	let (middle, rx_middle_man) = middle_man::Task::new(rx_encoder); // this is for testing purposes
	let (searcher, rx_search) = search_task::Task::new(rx_middle_man);
	let (decoder, rx_decode) = decode_task::Task::new(rx_search);

	let mut original_data = Vec::with_capacity(NUM_FRAMES);
	for _ in 0..NUM_FRAMES {
		original_data.push(random_bytes(SIZE_FRAME));
	}

	// start the tasks (any order works, but doing it in reverse is probably best)
	// decoder.start();
	searcher.start();
	middle.start();
	encoder.start();

	// send the data into the pipeline
	for each in &original_data {
		tx_start.send(each.clone()).unwrap();
	}
	drop(tx_start);

	let timer = time::Instant::now();
	decoder.start(); // starting the decoder here so we can get an accurate benchmark

	// receive the data from the pipeline
	let mut output_data: Vec<Bytes> = Vec::new();
	for _ in 0..NUM_FRAMES {
		output_data.push(rx_decode.recv().unwrap());
	}

	let dur = timer.elapsed();

	for (i, (og, out)) in original_data.iter().zip(output_data).enumerate() {
		if og.to_vec() != out.to_vec() {
			dbg!(i);
			assert_eq!(og.to_vec(), out.to_vec())
		}
	}
	
	dbg!(ACTUAL_BYTES);
	dbg!(dur);
}

fn random_bytes(len: usize) -> Bytes {
	let mut rng = rand::thread_rng();
	let mut data: Vec<u8> = vec![0; len];
	rng.fill_bytes(&mut data);

	data.into()
}

pub fn eprint_bin(arr: &[u8], tag: Option<&str>) {
	if let Some(s) = tag {
		eprintln!("{}:", s);
	}
	for b in arr {
		eprintln!("{:#010b}, {b:#3}", b);
	}
	eprintln!();
}

pub fn eprint_diff(arr1: &[u8], arr2: &[u8]) {
	let diff: Vec<u8> = arr1.iter().zip(arr2)
		.map(|x| {
			x.0 ^ x.1
		}).collect();

	eprint_bin(&diff, Some("diff"));
}

pub fn eprint_bytes_masked(arr: &[u8], mask: u8) {
	for b in arr {
		eprintln!("{}", b & mask);
	}
}