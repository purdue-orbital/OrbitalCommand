use std::thread::sleep;
use std::time::Duration;

use radio::radio_settings::ModulationTypes::BPSK;
use radio::radio_settings::RadioSetting;
use radio::runtime::Runtime;

static SAMPLE_RATE: f32 = 4e6;
static FREQUENCY: f32 = 916e6;
static BAUD_RATE: f32 = 1e4;

pub fn main() {
    let setting = RadioSetting::new(SAMPLE_RATE, BAUD_RATE, FREQUENCY, BPSK);

    let r = Runtime::new(setting).unwrap();

    sleep(Duration::from_secs(1));

    r.tx(&[127, 127, 127, 127]);

    sleep(Duration::from_secs(1));

    let arr = r.rx();

    dbg!(arr.clone());

    let mut test_arr = vec![0u8;4];

    for x in arr{
        for y in (0..8).rev() {
            let mut bit = (x >> y) & 1;

            for z in test_arr.iter_mut(){
                let hold = (*z >> 7) & 1;

                *z <<= 1;
                *z |= bit;

                bit = hold
            }

            if test_arr.as_slice() == [127, 127, 127, 127]{
             println!("Found!")
            }

        }

    }

}