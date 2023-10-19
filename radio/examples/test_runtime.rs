use std::thread::sleep;
use std::time::Duration;

use radio::radio_settings::ModulationTypes::BPSK;
use radio::radio_settings::RadioSetting;
use radio::runtime::Runtime;

static SAMPLE_RATE: f32 = 2e6;
static FREQUENCY: f32 = 916e6;
static BAUD_RATE: f32 = 1e4;

pub fn main() {
    let setting = RadioSetting::new(SAMPLE_RATE, BAUD_RATE, FREQUENCY, BPSK);

    let r = Runtime::new(setting).unwrap();

    r.tx(&[127, 127, 127, 127]);

    sleep(Duration::from_secs(1));

    let arr = r.rx();

    let test_arr = vec![0;4];

    for x in arr{
        for y in 0..8 {

        }

    }

}