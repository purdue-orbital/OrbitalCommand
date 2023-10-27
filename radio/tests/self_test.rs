extern crate radio;

use std::any::Any;
use std::thread;
use std::time::Duration;

// #[test]
// fn self_test_128bytes()
// {
//     let err = radio::RadioStream::new();
//
//     // if an error happens, most likely the radio is not connected so skip
//     if err.is_err() {
//         println!("Radio doesn't seem to be connected. Skipping radio test...");
//         return;
//     }
//
//     let stream = err.unwrap();
//
//     let mut test_arr: [u8; 4] = [4, 252, 112, 128];
//
//     // Allow for some delay
//     thread::sleep(Duration::from_secs(10));
//
//     // Transmit
//     stream.transmit(test_arr.as_mut_slice()).unwrap();
//
//     // Allow for some more delay
//     thread::sleep(Duration::from_secs(2));
//
//     // Read
//     let arr = stream.rx_buffer.read();
//
//     // Verify
//     assert_eq!(test_arr, arr.unwrap().as_slice());
//     assert_eq!(stream.settings.radio.clone().get_radio().unwrap().type_id(), stream.settings.radio.get_radio().unwrap().type_id());
//     assert!(stream.settings.radio.is_connected());
// }