use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Error, Result};
use num_complex::Complex;

use crate::dsp::{Demodulators, Modulators};
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx, Tx};
use crate::tools::bin_char_arr_to_usize_unchecked;

pub mod dsp;
mod radio;
mod tools;
mod streams;

/// Transmit at the top of x milliseconds (EX: x == 100, transmit and receive at every 100 milliseconds)
static TRANSMISSION_SYNC_TIME: usize = 200;

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

static AMBLE: &str = "101010101011110101110101010101010101010101010101010111111";

/// The Frame design implemented here is CCSDS SDLP which is specifically designed for use in
/// spacecraft and space bound communication
///
/// Here is the official standard: https://public.ccsds.org/Pubs/132x0b3.pdf
pub struct Frame {
    //--------------------------------
    // Transfer Frame Primary Header
    //--------------------------------

    // 2 bits
    Version_Number: u8,

    // 10 bits
    Spacecraft_ID: u16,

    // 3 bits
    Virtual_Channel_ID: u8,

    // 1 bits
    OCF: bool,

    // 8 bits
    Master_Frame_Count: u8,

    // 8 bits
    Virtual_Frame_Count: u8,

    // 16 bits
    Data_Status: u16,


    //--------------------------------
    // Main body
    //--------------------------------

    data:Vec<u8>

}

impl Frame {
    pub fn new(bytes: &[u8]) -> Frame {
        Frame { Version_Number: 0, Spacecraft_ID: 0, Virtual_Channel_ID: 0, OCF: false, Master_Frame_Count: 0, Virtual_Frame_Count: 0, Data_Status: 0, data: bytes.to_vec()}
    }

    /// Turn a string into frame segments (if any)
    pub fn from(data: &str) -> Vec<Frame>
    {
        // Create return vector
        let mut to_return = Vec::new();

        // remove "ambles"
        let clear = data.split(AMBLE).collect::<Vec<&str>>();

        for x in (1..clear.len()).step_by(2){
            to_return.push( Frame { Version_Number: 0, Spacecraft_ID: 0, Virtual_Channel_ID: 0, OCF: false, Master_Frame_Count: 0, Virtual_Frame_Count: 0, Data_Status: 0, data: bin_to_u8(clear[x])});
        }

        to_return
    }

    pub fn assemble(&self) -> String {

        let bin = u8_to_bin(self.data.as_slice());

        format!("{AMBLE}{bin}{AMBLE}")
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
            sample_rate: 1e5,
            lo_frequency: 916e6,
            lpf_filter: 0.0,
            channels_in_use: 0,
            gain: 50.0,
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
            modulation: Modulators::new(set.sample_rate as f32, set.baud_rate),
        };

        // Spawn rx thread
        spawn(move || {
            let mut rx_stream = Rx::new(set.clone()).expect("Starting RX stream");

            let mut instance = Demodulators::new(set.sample_rate as f32, set.baud_rate);

            // rx loop
            loop {

                // // Wait till time to listen
                while SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % TRANSMISSION_SYNC_TIME as u128 != 0{

                }

                let signal = rx_stream.fetch((set.clone().sample_rate * 0.2) as usize).expect("Reading stream");

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

        // Wait till time to transmit
        while SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % TRANSMISSION_SYNC_TIME as u128 != 0{

        }

        // add layer 2 data (frame header and trailer)
        let frame = Frame::new(data);

        // Modulate
        let signal = self.modulation.ask(frame.assemble().as_str());

            // Send
        self.tx_stream.send(signal.as_slice()).unwrap();


        Ok(())
    }

    /// This process samples read and return any data received
    pub fn read(&mut self) -> Result<Vec<Vec<u8>>> {

        // Read
        let s = self.rx_buffer.clone();
        // Turn Signal into frames
        let arr = Frame::from(s.lock().unwrap().as_str());

        // Turn frames into data and return the raw data
        let mut to_return = Vec::new();

        for x in arr {
            to_return.push(x.data)
        }

        // Clear buffer
        self.rx_buffer.lock().unwrap().clear();

        Ok(to_return)
    }
}

//--------------------------------------------------------------------------------------------------


/// This exposes functions for benchmarking
#[derive(Clone)]
pub struct Benchy {
    modulation: Arc<Mutex<Modulators>>,
    demodulation: Arc<Mutex<Demodulators>>,
}


impl Benchy {
    pub fn new() -> Benchy {
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

    // BPSK
    pub fn mod_bpsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().bpsk(bin)
    }
    pub fn demod_bpsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().bpsk(arr)
    }

    // QPSK
    pub fn mod_qpsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().qpsk(bin)
    }
    pub fn demod_qpsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().qpsk(arr)
    }

}


/// This exposes functions for testing
#[derive(Clone)]
pub struct Testy {
    modulation: Arc<Mutex<Modulators>>,
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

    // BPSK
    pub fn mod_bpsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().bpsk(bin)
    }
    pub fn demod_bpsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().bpsk(arr)
    }

    // QPSK
    pub fn mod_qpsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        self.modulation.lock().unwrap().qpsk(bin)
    }
    pub fn demod_qpsk(&mut self, arr: Vec<Complex<f32>>) -> String
    {
        self.demodulation.lock().unwrap().qpsk(arr)
    }

}