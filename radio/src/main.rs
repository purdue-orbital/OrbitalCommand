use crate::pipeline::Pipeline;
use std::thread;
use std::time::Duration;

mod pipeline;
mod radio;
mod stream;

fn main() {
    let _ = Pipeline::new(915e6, 100e3);

    thread::sleep(Duration::from_secs(10));
}
