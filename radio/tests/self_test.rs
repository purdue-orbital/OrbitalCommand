extern crate radio;

use std::thread;
use std::time::Duration;

#[test]
fn self_test_128bytes()
{
    let err = radio::RadioStream::new();

    // if an error happens, most likely the radio is not connected so skip
    if err.is_err() {
        println!("Radio doesn't seem to be connected. Skipping radio test...");
        return;
    }

    let mut stream = err.unwrap();

    let mut test_arr: [u8; 4] = [4, 252, 112, 128];

    // Allow for some delay
    thread::sleep(Duration::from_secs(1));

    // Transmit
    stream.transmit(test_arr.as_mut_slice()).expect("Transmit");

    // Allow for some more delay
    thread::sleep(Duration::from_secs(1));

    // Read
    let arr = stream.read().expect("Reading...");

    // Verify
    assert_eq!(test_arr, arr[0].as_slice())
}