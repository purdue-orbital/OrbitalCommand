use std::str;
use std::sync::{Arc, RwLock};
use std::thread::spawn;

fn main() {
    // Start Radio stream
    let check = radio::RadioStream::new();

    // Ensure radio is connected
    assert!(check.is_ok(), "Radio is not connected!");

    // Radio isn't thread safe due to soapysdr so we need to lock it
    let stream = Arc::new(RwLock::new(check.unwrap()));
    let thread_clone = stream.clone();

    // This is the thread we read transmissions from asynchronously
    spawn(move || {
        loop {
            // Read transmissions
            let arr = thread_clone.read().unwrap().read().unwrap();

            // Turn bytes into a string
            let check = str::from_utf8(arr.as_slice());

            if let Ok(..) = check {
                let out = check.unwrap().to_string();

                if !out.is_empty() {
                    println!("Data: {out}")
                }
            }
        }
    });

    // Start chat app
    loop {
        // allocate space for reading user input
        let mut line = String::new();

        // Take in user input
        std::io::stdin().read_line(&mut line).unwrap();

        // Send transmission
        stream.read().unwrap().transmit(line.as_bytes()).unwrap();
    }
}
