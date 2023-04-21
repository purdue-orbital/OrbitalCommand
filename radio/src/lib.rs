use anyhow::{Error, Result};
use num_complex::Complex;

use crate::dsp::{Demodulators, Modulators};
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx, Tx};

pub mod dsp;
mod radio;
mod tools;
mod streams;

pub struct Frame {
    data: Vec<u8>,
}

impl Frame {
    pub fn new(bytes: &[u8]) -> Frame {

        // Ensure that there is, at max, 7 bytes
        assert!(bytes.len() <= 7);

        Frame { data: Vec::from(bytes) }
    }

    pub fn assemble(&self) -> String {
        let mut name_in_binary = "".to_string();

        for character in self.data.clone() {
            name_in_binary += &format!("{:b}", character);
        }

        // Add preamble
        format!("11100001{}", name_in_binary)
    }
}

pub struct RadioStream {
    tx_stream: Tx,
    rx_stream: Rx,
    settings: RadioSettings,
}

impl RadioStream {
    /// This will create text into frame for transmission
    pub fn construct_frames(bytes: &[u8]) -> Vec<Frame> {
        let mut to_return = Vec::new();

        for x in 0..(bytes.len() / 7) {
            to_return.push(Frame::new(&bytes[(x * 7)..(x + 1) * 7]));
        }

        to_return
    }

    pub fn new() -> Result<RadioStream> {

        // Check if radio is connected
        let radio = Radio::new().unwrap();

        // Ensure radio is connected
        if !radio.is_connected() {
            return Err(Error::msg("Radio not connected!"));
        }

        // Radio settings
        let mut set = RadioSettings {
            sample_rate: 100e3,
            lo_frequency: 915e6,
            lpf_filter: 0.0,
            channels_in_use: 0,
            gain: 50.0,
            radio,
            baud_rate: 1e4,
            size: 0,
        };

        // Make radio streams
        let me = RadioStream {
            tx_stream: Tx::new(set.clone())?,
            rx_stream: Rx::new(set.clone())?,
            settings: set,
        };

        // Return
        Ok(me)
    }

    /// This will transmit binary data to the radio
    pub fn transmit(&mut self, bin: &str) -> Result<()> {

        // Modulate
        let signal = Modulators::ask(bin, self.settings.sample_rate, self.settings.baud_rate);

        // Send
        self.tx_stream.send(signal.as_slice())?;

        Ok(())
    }

    /// This process samples read and return any data received
    pub fn read(&mut self, bin: &str) -> Result<()> {

        // Modulate
        let signal = Modulators::ask(bin, self.settings.sample_rate, self.settings.baud_rate);

        // Send
        self.tx_stream.send(signal.as_slice())?;

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------


/// This exposes functions for benchmarking and testing
#[cfg(feature = "bench")]
pub struct Benchy {}


#[cfg(feature = "bench")]
impl Benchy {
    pub fn mod_ask<'a>(bin: &str, sample_rate: f64, baud_rate: f64) -> Vec<Complex<f32>>
    {
        Modulators::ask(bin, sample_rate, baud_rate)
    }

    pub fn demod_ask(arr: Vec<Complex<f32>>, sample_rate: f64, baud_rate: f64) -> String
    {
        Demodulators::ask(arr, sample_rate, baud_rate)
    }
}