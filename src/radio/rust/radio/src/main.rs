use std::thread;
use num_complex::Complex;
use soapysdr::Direction;

use plotters::prelude::*;
use radio::frequency_range;

use radio::dsp::generate_wave;
use radio::radio::Radio;
use radio::stream::Stream;
use radio::graphy;


fn main() {
    let lazy = frequency_range(915e6, 915.4e6);

    let radio = Radio::new().expect("Fetching radio");
    let mut s1 = Stream::new_tx(radio.clone(), 0, lazy.center_frequency, lazy.lpf_bandwidth, 300e3).expect("Get radio");
    let mut s2 = Stream::new_rx(radio, 1, lazy.center_frequency, lazy.lpf_bandwidth, 300e3).expect("Get radio");

    let mut arr = generate_wave(100e3, 300e3, 300);

    let hnd = thread::spawn(move || {
        for x in 0..100 {

            s2.rx();

        }

        let arr = s2.rx();
        graphy::graphy::graph("data.png", arr).unwrap();
    });


    //s1.tx(arr);


    // let mut arr = s2.rx();

    //let mut arr = GenerateWave(100e3, 300e3, 300 as i32);

    // graphy::graphy::graph("data.png", arr);
    hnd.join().unwrap();


}
