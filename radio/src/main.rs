use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::pipeline::Pipeline;

mod pipeline;
mod stream;
mod radio;
mod dsp;
mod tools;



fn main() {
    
    let mut pipe = Pipeline::new(915e6, 100e3).unwrap();


    sleep(Duration::from_secs_f32(15.0));

}
