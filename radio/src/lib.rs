#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard};
use std::thread::{sleep, spawn};
use std::time::Duration;

use anyhow::{Error, Result};
use num_complex::Complex;

use crate::dsp::{Demodulators, Modulators};
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx, Tx};

mod radio;
mod streams;
pub mod dsp;

static AMBLE: &str = "10101010";
static IDENT: &str = "1111000011110000";
static MOD_TYPE: ModulationType = ModulationType::BPSK;


enum ModulationType {
    ASK,
    FSK,
    MFSK,
    BPSK,
    QPSK,
}

fn bits_per_symbol() -> u8 {
    match MOD_TYPE {
        ModulationType::ASK => { 1 }
        ModulationType::FSK => { 1 }
        ModulationType::MFSK => { 8 }
        ModulationType::BPSK => { 1 }
        ModulationType::QPSK => { 2 }
    }
}

fn demodulation(obj: &Demodulators, arr: Vec<Complex<f32>>) -> Vec<u8> {
    match MOD_TYPE {
        ModulationType::ASK => { obj.ask(arr) }
        ModulationType::FSK => { obj.fsk(arr) }
        ModulationType::MFSK => { obj.mfsk(arr) }
        ModulationType::BPSK => { obj.bpsk(arr) }
        ModulationType::QPSK => { obj.qpsk(arr) }
    }
}

fn modulation(obj: &Modulators, arr: &[u8]) -> Vec<Complex<f32>> {
    match MOD_TYPE {
        ModulationType::ASK => { obj.ask(arr) }
        ModulationType::FSK => { obj.fsk(arr) }
        ModulationType::MFSK => { obj.mfsk(arr) }
        ModulationType::BPSK => { obj.bpsk(arr) }
        ModulationType::QPSK => { obj.qpsk(arr) }
    }
}


unsafe impl Send for RadioStream {}

unsafe impl Sync for RadioStream {}

/// u8 array to binary string
fn u8_to_bin(arr: &[u8]) -> String {
    let mut binary_string = String::new();

    for &byte in arr {
        let binary_byte = format!("{:08b}", byte);
        binary_string.push_str(&binary_byte);
    }

    binary_string
}

/// binary string to u8 array
fn bin_to_u8(bin: &str) -> Vec<u8> {
    let mut to_return = Vec::new();

    let mut hold = String::from("");

    let mut chars = bin.chars();

    // Split at every 8 digits ( to form 1 byte )
    for x in 0..bin.len() {

        if let Some(next_char) = chars.next(){
            hold.push(next_char)
        }

        if x % 8 == 7 {

            if let Ok(radix) = u8::from_str_radix(hold.as_str(), 2){
                to_return.push(radix);
            }

            hold.clear();
        }
    }

    to_return
}


/// The Frame design implemented here is CCSDS SDLP which is specifically designed for use in
/// spacecraft and space bound communication
///
/// Here is the official standard: https://public.ccsds.org/Pubs/132x0b3.pdf
pub struct Frame {
    //--------------------------------
    // Transfer Frame Primary Header
    //--------------------------------

    // 2 bits
    version_number: u8,

    // 10 bits
    spacecraft_id: u16,

    // 3 bits
    virtual_channel_id: u8,

    // 1 bits
    ocf: bool,

    // 8 bits
    master_frame_count: u8,

    // 8 bits
    virtual_frame_count: u8,

    // 16 bits
    data_status: u16,


    //--------------------------------
    // Main body
    //--------------------------------

    pub data: Vec<u8>,
}

impl Frame {
    pub fn new(bytes: &[u8]) -> Frame {
        Frame { version_number: 0, spacecraft_id: 0, virtual_channel_id: 0, ocf: false, master_frame_count: 0, virtual_frame_count: 0, data_status: 0, data: bytes.to_vec() }
    }

    /// Turn a string into frame segments (if any)
    pub fn from(data: Vec<String>) -> Vec<Frame>
    {
        // Create return vector
        let mut to_return = Vec::new();

        for x in data {
            to_return.push(Frame { version_number: 0, spacecraft_id: 0, virtual_channel_id: 0, ocf: false, master_frame_count: 0, virtual_frame_count: 0, data_status: 0, data: bin_to_u8(x.as_str()) });
        }

        to_return
    }

    pub fn assemble(&self) -> Vec<u8> {
        let bin = u8_to_bin(self.data.as_slice());

        let len = self.data.len() as u8;

        let len_bin = u8_to_bin(&[len]);

        bin_to_u8(format!("{AMBLE}{IDENT}{len_bin}{bin}").as_str())
    }
}

pub struct RadioStream {
    tx_stream: Tx,
    modulation: Modulators,
    rx_buffer: Arc<RwLock<Vec<Vec<u8>>>>,
    settings: RadioSettings,
}

struct RXLoop {
    len: usize,
    buffer: Arc<RwLock<Vec<String>>>,
    counter: usize,
    arr: [fn(rxloop: &mut RXLoop, window: &mut String) -> u8; 4],

}


impl RXLoop {
    pub fn new(buffer: Arc<RwLock<Vec<String>>>) -> RXLoop {
        RXLoop {
            len: 0,
            buffer,
            counter: 0,
            arr: [RXLoop::listen, RXLoop::sync, RXLoop::read_frame, RXLoop::record],
        }
    }

    pub fn run(&mut self, window: &mut String) {
        self.counter = (self.counter + self.arr[self.counter](self, window) as usize) % 4;
    }

    fn listen(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.contains('1') {
            1
        } else {
            window.clear();

            0
        }
    }

    fn sync(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.contains(IDENT)
        {
            window.clear();

            1
        } else if window.len() > 1000 {
            window.clear();

            3
        } else { 0 }
    }

    fn read_frame(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.len() >= 8 {
            rxloop.len = bin_to_u8(window.as_str())[0] as usize * 8usize;

            window.clear();

            1
        } else { 0 }
    }

    fn record(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.len() >= rxloop.len {
            if let Ok(mut write_buf) = rxloop.buffer.write(){

                write_buf.push(window.clone());
            }

            1
        } else {
            0
        }
    }
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
            sample_rate: 4e6,
            lo_frequency: 916e6,
            lpf_filter: 1e3,
            channels_in_use: 0,
            gain: 100.0,
            radio,
            baud_rate: 4e4,
            size: 0,
        };

        // Read buffer
        let buffer = Arc::new(RwLock::new(Vec::new()));

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
            if let Ok(mut rx_stream) = Rx::new(set.clone()){

                let samples_per_a_symbol = set.sample_rate as f32 / set.baud_rate;
                let instance = Demodulators::new(samples_per_a_symbol as usize, set.sample_rate as f32);

                // create mtu
                let mut mtu = vec![Complex::new(0.0, 0.0); samples_per_a_symbol as usize];

                // create window
                let mut window = "000000000000000000000000000000000000".to_string();

                let fake_buffer = Arc::new(RwLock::new(Vec::new()));

                let mut rxloop = RXLoop::new(fake_buffer.clone());

                // rx loop
                loop {
                    rxloop.run(&mut window);

                    let err = rx_stream.fetch(&[mtu.as_mut_slice()]);

                    if err.is_err() {}

                    if let Some(last_char) = u8_to_bin(demodulation(&instance, mtu.clone()).as_slice()).chars().last(){
                        window.push(last_char);
                    }


                    if let Ok(mut lock) = fake_buffer.write() {

                        if !lock.is_empty(){

                            let m = bin_to_u8(lock[0].as_str());

                            if let Ok(mut buf) = buffer.write() {
                                buf.push(m)
                            }

                            lock.clear();

                            window = "000000000000000000000000000000000000".to_string();

                        }
                    }
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
    pub fn read(&self) -> Result<Vec<u8>>{

        let mut stuff = if let Ok(stuff_to_clone) = self.rx_buffer.read(){
            stuff_to_clone.clone()
        }else {
            Vec::new()
        };



        while stuff.is_empty() {

            if let Ok(buff) = self.rx_buffer.read(){
                stuff = buff.clone()
            }

            sleep(Duration::from_millis(5))
        }

        // Clear buffer
        if let Ok(mut writeable) = self.rx_buffer.write(){

            writeable.clear();

            Ok(stuff[0].clone())

        }else {
            Err(Error::msg("Error trying to lock buffer to clear!"))
        }


    }

    pub fn receive_frames(&self) -> Result<Vec<Frame>> {
        if let Ok(bytes) = self.read(){
            Ok(Frame::from(vec![String::from_utf8(bytes)?]))
        }else {
            Err(Error::msg("Failed to read from stream!"))
        }
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
        Benchy { modulation: Arc::from(Mutex::from(Modulators::new(0, 0.0))), demodulation: Arc::from(Mutex::from(Demodulators::new(0, 0.0))) }
    }

    pub fn update(&mut self, sample_rate: f32, baud_rate: f32)->Result<(), Error> {


        if let Ok(mut mod_lock) = self.modulation.lock(){
            mod_lock.update((sample_rate / baud_rate) as usize, sample_rate);

            if let Ok(mut demod_lock) = self.demodulation.lock(){

                demod_lock.update((sample_rate / baud_rate) as usize, sample_rate);

                Ok(())
            }else {
                Err(
                    Error::msg("Unable to lock demodulator!")
                )
            }

        }else {
            Err(
                Error::msg("Unable to lock modulator!")
            )
        }
    }

    // ASK
    pub fn mod_ask(&mut self, bin: &[u8]) -> Result<Vec<Complex<f32>>, PoisonError<MutexGuard<'_, Modulators>>>
    {
        let test = self.modulation.lock()?;

        Ok(test.ask(bin))
    }
    pub fn demod_ask(&mut self, bin: Vec<Complex<f32>>) -> Result<Vec<u8>, PoisonError<MutexGuard<'_, Demodulators>>>
    {
        let test = self.demodulation.lock()?;

        Ok(test.ask(bin))
    }

    // FSK
    pub fn mod_fsk(&mut self, bin: &[u8]) -> Result<Vec<Complex<f32>>, PoisonError<MutexGuard<'_, Modulators>>>
    {
        let test = self.modulation.lock()?;

        Ok(test.fsk(bin))
    }
    pub fn demod_fsk(&mut self, bin: Vec<Complex<f32>>) -> Result<Vec<u8>, PoisonError<MutexGuard<'_, Demodulators>>>
    {
        let test = self.demodulation.lock()?;

        Ok(test.fsk(bin))
    }
    // MFSK
    pub fn mod_mfsk(&mut self, bin: &[u8]) -> Result<Vec<Complex<f32>>, PoisonError<MutexGuard<'_, Modulators>>>
    {
        let test = self.modulation.lock()?;

        Ok(test.mfsk(bin))
    }
    pub fn demod_mfsk(&mut self, bin: Vec<Complex<f32>>) -> Result<Vec<u8>, PoisonError<MutexGuard<'_, Demodulators>>>
    {
        let test = self.demodulation.lock()?;

        Ok(test.mfsk(bin))
    }

    // BPSK
    pub fn mod_bpsk(&mut self, bin: &[u8]) -> Result<Vec<Complex<f32>>, PoisonError<MutexGuard<'_, Modulators>>>
    {
        let test = self.modulation.lock()?;

        Ok(test.bpsk(bin))
    }
    pub fn demod_bpsk(&mut self, bin: Vec<Complex<f32>>) -> Result<Vec<u8>, PoisonError<MutexGuard<'_, Demodulators>>>
    {
        let test = self.demodulation.lock()?;

        Ok(test.bpsk(bin))
    }

    // QPSK
    pub fn mod_qpsk(&mut self, bin: &[u8]) -> Result<Vec<Complex<f32>>, PoisonError<MutexGuard<'_, Modulators>>>
    {
        let test = self.modulation.lock()?;

        Ok(test.qpsk(bin))
    }

    pub fn demod_qpsk(&mut self, bin: Vec<Complex<f32>>) -> Result<Vec<u8>, PoisonError<MutexGuard<'_, Demodulators>>>
    {
        let test = self.demodulation.lock()?;

        Ok(test.qpsk(bin))
    }
}