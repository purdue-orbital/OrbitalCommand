use std::{str, thread};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

fn main() {
    // Start Radio stream
    let check = radio::RadioStream::new();

    // Ensure radio is connected
    assert!(check.is_ok(), "Radio is not connected!");

    // Radio isn't thread safe due to soapysdr so we need to lock it
    let stream = Arc::new(Mutex::new(check.unwrap()));
    let thread_clone = stream.clone();

    // This is the thread we read transmissions from asynchronously
    spawn(move || {

        loop {

            // Read transmissions
            let arr = thread_clone.lock().unwrap().read().unwrap();

            // Loop through each transmission received
            for x in arr {

                // Turn bytes into a string
                let check = str::from_utf8(x.as_slice());

                if let Ok(..) = check {
                    let out = check.unwrap().to_string();

                    println!("{out}")
                }
            }

            // Wait for more transmissions
            thread::sleep(Duration::from_secs(1));
        }
    });


    // Start chat app
    loop {
        // allocate space for reading user input
        let mut line = String::new();

        // Take in user input
        std::io::stdin().read_line(&mut line).unwrap();

        // Send transmission
        stream.lock().unwrap().transmit(line.as_bytes()).unwrap();
    }
}