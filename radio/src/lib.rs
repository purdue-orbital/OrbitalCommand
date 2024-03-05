use std::thread::spawn;
use num_complex::Complex;
use rustdsp::elements::builder::PipelineBuilder;
use rustdsp::elements::data_trigger::DataTriggerComplex;
use rustdsp::elements::waterfall_chart::WaterfallChart;
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx};

mod streams;
mod radio;

pub struct OrbitalRadio {
    
}

impl OrbitalRadio {
    pub fn new(){

        let settings = RadioSettings{
            sample_rate: 1e6,
            lo_frequency: 916e6,
            lpf_filter: 0.0,
            channels_in_use: 0,
            gain: 20.0,
            radio: Radio::new().unwrap(),
            baud_rate: 100.0,
            size: 0,
        };
        
        let mut r = Rx::new(settings).unwrap();

        let (trigger,tx) = DataTriggerComplex::new(16);
        
        let mut builder = PipelineBuilder::new();
        builder.add(trigger);
        builder.add(WaterfallChart::new());
        
        spawn(move ||{
            let mut binding = vec![Complex::new(0.0,0.0);16];
            let complex = binding.as_mut_slice();
            
            loop{
                r.fetch(&mut [complex]).unwrap();
                tx.send(complex.to_vec()).unwrap();
            }
        });
        
        spawn(move || {
            
        });
        
        builder.build().run();

        loop {
            
        }
    }
}