use std::thread;
use std::time::Duration;
use crate::pipeline::Pipeline;

mod pipeline;
mod stream;
mod radio;



fn main() {
    let mut pipe = Pipeline::new(915e6, 100e3);

    thread::sleep(Duration::from_secs(10));
}
