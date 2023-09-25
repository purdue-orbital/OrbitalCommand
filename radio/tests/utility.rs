use std::sync::{Arc, RwLock};

use radio::{AMBLE, IDENT, runtime};
use radio::dsp::Modulators;
use radio::frame::Frame;

/// u8 array to binary string
fn u8_to_bin(arr: &[u8]) -> String {
    let mut name_in_binary = String::from("");

    for character in arr {
        name_in_binary += &format!("{:08b}", *character);
    }

    name_in_binary
}

#[test]
fn u8_to_bin_test() {
    let bin = [3_u8, 5, 1, 2];
    let expected = "00000011000001010000000100000010".to_string();

    let to_test = u8_to_bin(bin.as_slice());

    assert_eq!(to_test, expected, "u8 to bin check.\n\tGot: {}\n\tExpected: {}", to_test, expected);
}

#[test]
fn frame_test() {

    // Test bytes
    let test_arr1 = [4, 252, 112, 128];

    // Make a frame
    let frame_1 = radio::frame::Frame::new(test_arr1.clone().as_mut_slice());

    // Turn the frame into a string
    let for_transmission1 = frame_1.assemble();

    println!("{:?}", &for_transmission1[(AMBLE.len() / 8)..]);

    // Reassemble
    let frame_3 = radio::frame::Frame::from(&for_transmission1[(AMBLE.len() / 8)..]);

    println!("{}", frame_3.has_ident);

    // Ensure frames match
    assert_eq!(frame_1.assemble(), frame_3.assemble());
}

#[test]
fn simulated_live_test() {
    // data settings
    let sample_rate = 20e6;
    let baud_rate = 2e4;
    let samples_per_symbol = sample_rate / baud_rate;

    // create simulated read loop
    let fake_buffer = Arc::new(RwLock::new(vec![]));
    let mut r = runtime::Runtime::new(samples_per_symbol as usize, sample_rate, IDENT, fake_buffer.clone());

    // create simulated data
    let test_data = vec![56, 203, 1, 0, 69];
    let m = Modulators::new(samples_per_symbol as usize, sample_rate);
    let test_frame = Frame::new(test_data.as_slice());
    let test_data_moded = m.bpsk(test_frame.assemble().as_slice());

    for x in (0..test_data_moded.len()).step_by(samples_per_symbol as usize) {
        r.run(test_data_moded[x..x + samples_per_symbol as usize].to_vec());
    }

    println!("{}", (test_data_moded.len() / samples_per_symbol as usize) / 16);

    assert!(!fake_buffer.read().unwrap().is_empty());
    assert_eq!(fake_buffer.read().unwrap()[0], test_data);
}