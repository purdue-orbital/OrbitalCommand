#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![doc(html_logo_url = "https://images.squarespace-cdn.com/content/v1/56ce2044d210b8716143af3a/1521674398338-46K9U3FDEZYYAYFJU6SG/Logo2.png", html_favicon_url = "https://images.squarespace-cdn.com/content/v1/56ce2044d210b8716143af3a/1521674398338-46K9U3FDEZYYAYFJU6SG/Logo2.png")]
//#![deny(missing_docs)]

use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn};
use std::time::Duration;

use anyhow::{Error, Result};
use num_complex::Complex;

use crate::dsp::{Demodulators, Modulators};
use crate::frame::Frame;
use crate::radio::Radio;
use crate::rx_handling::{RXLoop, WindowHandler};
use crate::streams::{RadioSettings, Rx, Tx};


mod radio;
mod streams;
pub mod dsp;
pub mod frame;
pub mod tools;
pub mod rx_handling;

pub static AMBLE: &str = "10101010101010101010101010101010";
pub static IDENT: &str = "11110000111100001111000011110000";
pub static MOD_TYPE: ModulationType = ModulationType::BPSK;

pub enum ModulationType {
    ASK,
    FSK,
    BPSK,
    QPSK,
}

pub fn bits_per_symbol() -> u8 {
    match MOD_TYPE {
        ModulationType::ASK => { 1 }
        ModulationType::FSK => { 1 }
        ModulationType::BPSK => { 1 }
        ModulationType::QPSK => { 2 }
    }
}

pub fn demodulation(obj: &Demodulators, arr: Vec<Complex<f32>>) -> Vec<u8> {
    match MOD_TYPE {
        ModulationType::ASK => { obj.ask(arr) }
        ModulationType::FSK => { obj.fsk(arr) }
        ModulationType::BPSK => { obj.bpsk(arr) }
        ModulationType::QPSK => { obj.qpsk(arr) }
    }
}

pub fn modulation(obj: &Modulators, arr: &[u8]) -> Vec<Complex<f32>> {
    match MOD_TYPE {
        ModulationType::ASK => { obj.ask(arr) }
        ModulationType::FSK => { obj.fsk(arr) }
        ModulationType::BPSK => { obj.bpsk(arr) }
        ModulationType::QPSK => { obj.qpsk(arr) }
    }
}


unsafe impl Send for RadioStream {}

unsafe impl Sync for RadioStream {}


pub struct RadioStream {
    pub tx_stream: Tx,
    pub modulation: Modulators,
    pub rx_buffer: Arc<RwLock<Vec<Vec<u8>>>>,
    pub settings: RadioSettings,
}


impl RadioStream {
    pub fn new() -> Result<RadioStream> {

        // Check if radio is connected
        let radio = Radio::new()?;

        // Ensure radio is connected
        if !radio.is_connected() {
            return Err(Error::msg("Radio not connected!"));
        }

        // Radio settings
        let set = RadioSettings {
            sample_rate: 2e7,
            lo_frequency: 916e6,
            lpf_filter: 1e3,
            channels_in_use: 0,
            gain: 100.0,
            radio,
            baud_rate: 2e5,
            size: 0,
        };

        // Read buffer
        let buffer = Arc::new(RwLock::new(Vec::with_capacity(20)));

        // Make radio streams
        let me = RadioStream {
            tx_stream: Tx::new(set.clone())?,
            rx_buffer: buffer.clone(),
            settings: set.clone(),
            modulation: Modulators::new((set.sample_rate / set.baud_rate as f64) as usize, set.sample_rate as f32),
        };


        // Spawn rx thread
        spawn(move || {
            // create stream
            if let Ok(mut rx_stream) = Rx::new(set.clone()) {
                let samples_per_a_symbol = set.sample_rate as f32 / set.baud_rate;
                let instance = Demodulators::new(samples_per_a_symbol as usize, set.sample_rate as f32);

                // create mtu
                let mut mtu = vec![Complex::new(0.0, 0.0); samples_per_a_symbol as usize];

                // create window
                let mut window = WindowHandler::new(IDENT);

                let mut rxloop = RXLoop::new(buffer);

                // rx loop
                loop {
                    rxloop.run(&mut window);

                    let err = rx_stream.fetch(&[mtu.as_mut_slice()]);

                    if err.is_err() {
                        println!("Error!")
                    }

                    window.add(demodulation(&instance,mtu.clone()).as_slice());

                }
            }
        });

        // Return
        Ok(me)
    }

    /// This will transmit binary data to the radio
    pub fn transmit(&self, data: &[u8]) -> Result<()> {

        // add layer 2 data (frame header and trailer)
        let frame = Frame::new(data);

        // Modulate
        let signal = modulation(&self.modulation, frame.assemble().as_slice());

        // Send
        self.tx_stream.send(signal.as_slice())?;

        Ok(())
    }

    pub fn transmit_frame(&self, frame: &Frame) -> Result<()> {
        self.transmit(&frame.assemble())
    }

    /// This process samples read and return any data received
    pub fn read(&self) -> Result<Vec<u8>> {
        let mut stuff = if let Ok(stuff_to_clone) = self.rx_buffer.read() {
            stuff_to_clone.clone()
        } else {
            Vec::new()
        };

        while stuff.is_empty() {
            if let Ok(buff) = self.rx_buffer.read() {
                stuff = buff.clone()
            }

            sleep(Duration::from_millis(5))
        }

        // Clear buffer
        if let Ok(mut writeable) = self.rx_buffer.write() {
            writeable.clear();

            Ok(stuff[0].clone())
        } else {
            Err(Error::msg("Error trying to lock buffer to clear!"))
        }
    }

    pub fn receive_frames(&self) -> Result<Vec<Frame>> {
        if let Ok(bytes) = self.read() {
            Ok(Frame::from(vec![String::from_utf8(bytes)?]))
        } else {
            Err(Error::msg("Failed to read from stream!"))
        }
    }
}
