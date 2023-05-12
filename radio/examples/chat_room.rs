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

        // This will hold message history per a read and remove redundant transmissions
        let mut hold = Vec::new();

        loop {

            // Read transmissions
            let arr = thread_clone.lock().unwrap().read().unwrap();

            // Loop through each transmission received
            for x in arr {

                // Turn bytes into a string
                let check = str::from_utf8(x.as_slice());

                if check.is_ok() {
                    let out = check.unwrap().to_string();

                    // check to see if string is in hold
                    if !hold.contains(&out) {

                        // if not print and add to hold
                        println!("{}", out.as_str());

                        hold.push(out);
                    }
                }
            }

            // clear hold for next batch
            hold.clear();

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

        // Radio doesn't yet have the ability to tell frames are incomplete so they just get dropped.
        // To compensate for now, we just spam transmissions and remove the redundant ones
        for _ in 0..5 {
            // preform delay so we don't lose all the packets
            thread::sleep(Duration::from_millis(100));

            // Send transmission
            stream.lock().unwrap().transmit(line.as_bytes()).unwrap();
        }
    }
}