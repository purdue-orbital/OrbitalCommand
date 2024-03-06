use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use num_complex::Complex;
use rustdsp::elements::builder::PipelineBuilder;
use rustdsp::elements::data_trigger::{DataTrigger, DataTriggerComplex};
use rustdsp::elements::events::Event;
use rustdsp::elements::frequency_demodulation::FrequencyDemodulation;
use rustdsp::elements::frequency_modulation::FrequencyModulation;
use rustdsp::elements::pub_sub::PubSub;
use rustdsp::math::objects::ElementParameter;
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx, Tx};

mod streams;
mod radio;

pub struct RxEvent{
    tx: Sender<Vec<f32>>,
}

impl RxEvent{
    pub fn new() -> (RxEvent, Receiver<Vec<f32>>){
        let (tx, rx) = channel();

        (RxEvent{
            tx
        }, rx)
    }
}

impl Event for RxEvent {
    fn run(&mut self, samples: &mut ElementParameter) -> bool {
        
        self.tx.send(samples.get_f32_array()).unwrap();
        
        true
    }
}


pub struct TxEvent{
    tx: Sender<Vec<Complex<f32>>>,
}

impl TxEvent {
    pub fn new() -> (TxEvent, Receiver<Vec<Complex<f32>>>){
        let (tx, rx) = channel();

        (TxEvent {
            tx
        }, rx)
    }
}

impl Event for TxEvent {
    fn run(&mut self, samples: &mut ElementParameter) -> bool {

        self.tx.send(samples.get_complex_f32().to_vec()).unwrap();

        true
    }
}

pub struct OrbitalRadio {
    tx_setting: RadioSettings,
    rx_setting: RadioSettings,
    rx: Receiver<Vec<f32>>,
    tx: Sender<Vec<f32>>
}

impl Default for OrbitalRadio{
    fn default() -> Self {
        
        let radio = Radio::new().unwrap();
        
        // try to load the radio settings
        let mut rx_setting_result = RadioSettings::load_from_file("rx_setting.toml");
        let mut tx_setting_result = RadioSettings::load_from_file("tx_setting.toml");
        
        if rx_setting_result.is_none(){
            rx_setting_result = Some(RadioSettings {
                sample_rate: 1e6,
                lo_frequency: 916e6,
                channels_in_use: 0,
                gain: 20.0,
                baud_rate: 100.0,
                sps: 16,
            });
        }
        
        if tx_setting_result.is_none(){
            tx_setting_result = Some(RadioSettings {
                sample_rate: 1e6,
                lo_frequency: 916e6,
                channels_in_use: 0,
                gain: 20.0,
                baud_rate: 100.0,
                sps: 16,
            });
        }

        let rx_setting = rx_setting_result.unwrap();
        rx_setting.save_to_file("rx_setting.toml");

        let tx_setting = tx_setting_result.unwrap();
        tx_setting.save_to_file("tx_setting.toml");
        
        let setting_copy_rx = rx_setting.clone();
        let setting_copy_tx = tx_setting.clone();
        
        let (rx_element, rx) = RxEvent::new();
        let (tx_element, rx_transmit) = TxEvent::new();

        // create data trigger for transmissions
        let (tx_trigger,tx) = DataTrigger::new();


        // create streams
        let mut r = Rx::new(radio.clone(),setting_copy_rx.clone()).unwrap();
        
        // rx thread
        spawn(move ||{
            
            // create data trigger
            let (trigger,tx) = DataTriggerComplex::new(16);

            // create workflow
            let mut builder = PipelineBuilder::new();
            builder.add(trigger);
            builder.add(FrequencyDemodulation::new(setting_copy_rx.sps, 1e3, setting_copy_rx.sample_rate as f32,setting_copy_rx.sps as f32 / 8.0));
            builder.add(PubSub::new(rx_element));
            
            // run
            spawn(move ||{
                builder.build().run();
            });
                

            let mut binding = vec![Complex::new(0.0,0.0); setting_copy_rx.sps];
            let complex = binding.as_mut_slice();

            loop{
                r.fetch(&mut [complex]).unwrap();
                tx.send(complex.to_vec()).unwrap();
            }
        });

        spawn(move || {
            let t = Tx::new(radio,setting_copy_tx.clone()).unwrap();

            // create workflow
            let mut builder = PipelineBuilder::new();
            builder.add(tx_trigger);
            builder.add(FrequencyModulation::new(setting_copy_tx.sps, 1e3, setting_copy_tx.sample_rate as f32));
            builder.add(PubSub::new(tx_element));

            // run
            spawn(move ||{
                builder.build().run();
            });

            loop{
                let mut data= rx_transmit.recv().unwrap();
                t.send(data.as_mut_slice()).unwrap();
            }
        });
        
        OrbitalRadio { tx_setting, rx_setting, rx, tx }
    }
}

impl OrbitalRadio {

    pub fn send(&self, data: &[f32]){
        self.tx.send(data.to_vec()).unwrap()
    }

    pub fn fetch(&self) -> Vec<f32> {
        self.rx.recv().unwrap()
    }
}