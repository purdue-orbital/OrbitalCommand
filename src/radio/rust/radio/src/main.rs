use std::{io, thread};
use std::io::{Read, stdin, stdout, Write};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use num_complex::Complex;
use soapysdr::Direction;

use plotters::prelude::*;
use radio::frequency_range;

use radio::dsp::generate_wave;
use radio::radio::Radio;
use radio::stream::Stream;
use radio::graphy;


fn main() {
    let lazy = frequency_range(915e6, 916e6);
    let mut sample_rate = 100e3;

    let radio = Radio::new().expect("Fetching radio");
    let mut s1 = Stream::new_tx(radio.clone(), 0, lazy.center_frequency, lazy.lpf_bandwidth, sample_rate).expect("Get radio");
    let mut s2 = Stream::new_rx(radio, 1, lazy.center_frequency, lazy.lpf_bandwidth, sample_rate).expect("Get radio");

    let mut arr = generate_wave(1000.0, sample_rate, 50);



    let hnd = thread::spawn(move || {
        while true
        {
            while SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() % 1000 != 0 {}

            // Get last set of data
            for _ in 0..100 {
                s2.rx();
            }

            let mut arr = s2.rx();

            s2.clear_buffer();

            thread::spawn(move || {

                // prepare date
                let mut avg_over_time = Vec::new();
                let mut to_avg = Vec::new();
                let avg_length = 1000;
                let mut to_avg_num = 0.0;

                // Average the amplitudes
                for x in 0..arr.len() - 1
                {
                    to_avg.push((arr.get(x as usize).unwrap().re.powf(2.0) + arr.get(x as usize).unwrap().im.powf(2.0)).sqrt());

                    if (x > avg_length)
                    {
                        let mut num = 0.0;

                        // add data to be averaged
                        for y in to_avg.clone()
                        {
                            num += 300.0 * y;
                        }

                        avg_over_time.push(num / avg_length as f32);

                        to_avg.remove(0 as usize);
                    }
                }

                // calculate the average of the averages
                for x in avg_over_time.clone()
                {
                    to_avg_num += x;
                }
                let total_avg = to_avg_num / avg_over_time.len() as f32;

                // drop averages down closer to zero and remove data that is below the average
                for x in 0..avg_over_time.len()
                {
                    let mut i = (*avg_over_time.get(x).unwrap()) - total_avg;

                    i *= (i > 0.0) as i32 as f32;

                    avg_over_time[x] = i;
                }

                let mut counter = 0;
                let mut last_counter = 0;
                let mut bin = "".to_owned();

                while counter < avg_over_time.len()
                {
                    if avg_over_time[counter] > 0.05
                    {
                        if counter - last_counter > 10
                        {
                            let mut hold = (counter - last_counter) as i32;

                            hold -= 3300;

                            while hold > 0
                            {
                                bin.push('0');
                                hold -= 3300;
                            }

                            bin.push('1');
                        }
                        last_counter = counter;
                    }

                    counter += 1;
                }

                graphy::graphy::graph_vec("data.png", avg_over_time).unwrap();

            });
        }
    });

    let mut test = "111111";

    //loop
    {
        //while SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() % 1000 != 0 {}

        for x in test.chars()
        {
            sleep(Duration::from_micros(900));

            if x == '1'
            {
                s1.tx(arr.clone());
            }
        }
    }

    // graphy::graphy::graph("data.png", arr);
    hnd.join().unwrap();


}
