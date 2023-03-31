use std::thread;
use std::time::Duration;
use crate::pipeline::Pipeline;

mod pipeline;
mod stream;
mod radio;
mod dsp;
mod tools;



fn main() {
    let mut s = "111000111";

    let mut modded = dsp::Modulators::fsk(s, 100e3, (50.0 / 100e3));
    let mut demodded = dsp::Demodulators::fsk(modded.clone(), 100e3, (50.0 / 100e3));

    println!("{}", demodded);

    dsp::Graph::time_graph("data.png", modded.clone()).unwrap();


}
