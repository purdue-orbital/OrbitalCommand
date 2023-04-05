use std::thread;
use std::thread::sleep;
use std::time::Duration;
use radio::pipeline::Pipeline;
use crate::pipeline::Pipeline;



fn main() {
    
    let mut pipe = Pipeline::new(915e6, 100e3).unwrap();


    sleep(Duration::from_secs_f32(15.0));

}
