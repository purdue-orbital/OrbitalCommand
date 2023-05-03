use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use anyhow::{Error, Result};
use num_complex::Complex;

use crate::dsp::{Demodulators, Modulators};
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx, Tx};

pub mod dsp;
mod radio;
mod tools;
mod streams;

static TRANSMISSION_SIZE: usize = 100;
// Maximum number of bytes to transmit per a frame
static PREAMBLE: [u8; 4] = [16, 32, 64, 128];
// Start of transmission sequence
static END: [u8; 4] = [255, 69, 55, 2]; // End of transmission sequence


/// u8 array to binary string
fn u8_to_bin(arr: &[u8]) -> String {
    let mut name_in_binary = String::from("");

    for character in arr {
        name_in_binary += &format!("{:08b}", *character);
    }

    name_in_binary
}

/// binary string to u8 array
fn bin_to_u8(bin: &str) -> Vec<u8> {
    let mut to_return = Vec::new();

    let mut hold = String::from("");

    let mut chars = bin.chars();

    // Split at every 8 digits ( to form 1 byte )
    for x in 0..bin.len() {
        hold.push(chars.next().unwrap());

        if x % 8 == 7 {
            to_return.push(u8::from_str_radix(hold.as_str(), 2).unwrap());

            hold.clear();
        }
    }

    to_return
}


pub struct Frame {
    data: Vec<u8>,
}

impl Frame {
    pub fn new(bytes: &[u8]) -> Frame {

        // Ensure transmission size
        //assert!(bytes.len() <= TRANSMISSION_SIZE);

        Frame { data: Vec::from(bytes) }
    }

    /// Turn a string into frame segments if any
    pub fn from(data: &str) -> Vec<Frame>
    {
        // Create return vector
        let mut to_return = Vec::new();

        // Make the preamble and post-amble bytes into binary strings
        let pre = u8_to_bin(PREAMBLE.clone().as_mut_slice());
        let post = u8_to_bin(END.clone().as_mut_slice());

        let part_1: Vec<&str> = data.split(pre.as_str()).collect();

        for x in part_1 {
            let test: Vec<&str> = x.split(post.as_str()).collect();

            if test.len() == 2 {
                to_return.push(Frame::new(bin_to_u8(test[0]).as_mut_slice()));
            }
        }


        to_return
    }

    pub fn assemble(&self) -> String {
        let pre = u8_to_bin(PREAMBLE.clone().as_mut_slice());
        let body = u8_to_bin(self.data.clone().as_mut_slice());
        let post = u8_to_bin(END.clone().as_mut_slice());

        format!("{pre}{body}{post}")
    }
}

pub struct RadioStream {
    tx_stream: Tx,
    modulation: Modulators,
    rx_buffer: Arc<Mutex<String>>,
    settings: RadioSettings,
}


impl RadioStream {
    pub fn new() -> Result<RadioStream> {

        // Check if radio is connected
        let radio = Radio::new().unwrap();

        // Ensure radio is connected
        if !radio.is_connected() {
            return Err(Error::msg("Radio not connected!"));
        }

        // Radio settings
        let set = RadioSettings {
            sample_rate: 100e3,
            lo_frequency: 430e6,
            lpf_filter: 0.0,
            channels_in_use: 0,
            gain: 1e3,
            radio,
            baud_rate: 1e4,
            size: 0,
        };

        // Read buffer
        let buffer = Arc::new(Mutex::new(String::from("")));

        // Make radio streams
        let me = RadioStream {
            tx_stream: Tx::new(set.clone())?,
            rx_buffer: buffer.clone(),
            settings: set.clone(),
            modulation: Modulators::new(set.sample_rate as f32,set.baud_rate),
        };

        // Spawn rx thread
        spawn(move || {
            let mut rx_stream = Rx::new(set.clone()).expect("Starting RX stream");

            let mut instance = Demodulators::new(set.sample_rate as f32, set.baud_rate);

            // rx loop
            loop {
                let signal = rx_stream.fetch((set.clone().sample_rate * 0.5) as usize).expect("Reading stream");

                let demod = instance.ask(signal);

                let mut data = buffer.lock().unwrap();
                *data = format!("{}{}", *data, demod);
            }
        });

        // Return
        Ok(me)
    }

    /// This will transmit binary data to the radio
    pub fn transmit(&mut self, data: &[u8]) -> Result<()> {

        // Break bytes into multiple frames if needed
        let arr = data.chunks(TRANSMISSION_SIZE - 1);

        // Go through each chunk and transmit
        for x in arr {

            // bytes into frames
            let frame = Frame::new(x);

            // Modulate
            let signal = self.modulation.ask(frame.assemble().as_str());

            // Send
            self.tx_stream.send(signal.as_slice()).unwrap();
        }

        Ok(())
    }

    /// This process samples read and return any data received
    pub fn read(&mut self) -> Result<Vec<Vec<u8>>> {

        // Read
        let s = self.rx_buffer.clone();

        // Clear buffer
        self.rx_buffer.lock().unwrap().clear();

        // Turn Signal into frames
        let arr = Frame::from(s.lock().unwrap().as_str());

        // Turn frames into data and return the raw data
        let mut to_return = Vec::new();

        for x in arr {
            to_return.push(x.data);
        }

        Ok(to_return)
    }
}

//--------------------------------------------------------------------------------------------------


/// This exposes functions for benchmarking
#[cfg(feature = "bench")]
#[derive(Clone)]
pub struct Benchy {
    modulation:  Arc<Mutex<Modulators>>,
    demodulation: Arc<Mutex<Demodulators>>,
}


#[cfg(feature = "bench")]
impl Benchy {
    pub fn new() -> Benchy{
        Benchy { modulation: Arc::from(Mutex::from(Modulators::new(0.0, 0.0))), demodulation: Arc::from(Mutex::from(Demodulators::new(0.0, 0.0))) }
    }

    pub fn update(&mut self, sample_rate: f32, baud_rate: f32) {
        self.modulation.lock().unwrap().update(sample_rate, baud_rate);
        self.demodulation.lock().unwrap().update(sample_rate, baud_rate);
    }

    // ASK
    pub fn mod_ask(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().ask(bin)
    }
    pub fn demod_ask(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().ask(arr)
    }

    // FSK
    pub fn mod_fsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().fsk(bin)
    }
    pub fn demod_fsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().fsk(arr)
    }

    // MFSK
    pub fn mod_mfsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().mfsk(bin)
    }
    pub fn demod_mfsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().mfsk(arr)
    }
}


/// This exposes functions for testing
#[derive(Clone)]
pub struct Testy {
    modulation:  Arc<Mutex<Modulators>>,
    demodulation: Arc<Mutex<Demodulators>>,
}

impl Testy {
    pub fn new() -> Testy {
        Testy { modulation: Arc::from(Mutex::from(Modulators::new(0.0, 0.0))), demodulation: Arc::from(Mutex::from(Demodulators::new(0.0, 0.0))) }
    }

    pub fn update(&mut self, sample_rate: f32, baud_rate: f32) {
        self.modulation.lock().unwrap().update(sample_rate, baud_rate);
        self.demodulation.lock().unwrap().update(sample_rate, baud_rate);
    }

    // ASK
    pub fn mod_ask(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().ask(bin)
    }
    pub fn demod_ask(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().ask(arr)
    }

    // FSK
    pub fn mod_fsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().fsk(bin)
    }
    pub fn demod_fsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().fsk(arr)
    }

    // MFSK
    pub fn mod_mfsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().mfsk(bin)
    }
    pub fn demod_mfsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().mfsk(arr)
    }
}