use num_complex::Complex;
use soapysdr::Direction;
use std::io::{stdin, stdout, Read, Write};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::{io, thread};

use radio::{frequency_range, RadioReader};

use radio::dsp::generate_wave;
use radio::graphy;
use radio::radio::Radio;
use radio::stream::Stream;

fn main() {
    let lazy = frequency_range(915e6, 916e6);
    let mut sample_rate = 100e3;

    let radio = Radio::new().expect("Fetching radio");
    let mut s1 = Stream::new_tx(
        radio.clone(),
        0,
        lazy.center_frequency,
        lazy.lpf_bandwidth,
        sample_rate,
    )
    .expect("Get radio");
    let mut s2 = Stream::new_rx(
        radio,
        1,
        lazy.center_frequency,
        lazy.lpf_bandwidth,
        sample_rate,
    )
    .expect("Get radio");

    let mut arr = generate_wave(1000.0, sample_rate, 50);

    let hnd = thread::spawn(move || {
        let mut rr = RadioReader::new(s2);

        rr.read();

        sleep(Duration::from_secs(30));
    });

    let mut test = "1010101010011001010101010101010101010101001010101010";

    //loop
    {
        //while SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() % 1000 != 0 {}

        for x in test.chars() {
            sleep(Duration::from_micros(900));

            if x == '1' {
                s1.tx(arr.clone().as_slice());
            }
        }
    }



    // graphy::graphy::graph("data.png", arr);
    hnd.join().unwrap();
}
