#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![doc(html_logo_url = "https://images.squarespace-cdn.com/content/v1/56ce2044d210b8716143af3a/1521674398338-46K9U3FDEZYYAYFJU6SG/Logo2.png", html_favicon_url = "https://images.squarespace-cdn.com/content/v1/56ce2044d210b8716143af3a/1521674398338-46K9U3FDEZYYAYFJU6SG/Logo2.png")]
//#![deny(missing_docs)]

//! This crate utilizes SDRs for long range communications intended for space bound communications.
//! This crate will allow for building more code on top of and make it easier for networking based
//! protocols to be developed and implemented with use of an SDR.
//!
//! It should be noted that any library that uses this crate will need to run with root privileges
//! or else the system won't be able to connect to the radio.


use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn};
use std::time::Duration;

use anyhow::{Error, Result};
use num_complex::Complex;
use rustdsp::{Demodulators, Modulators};

use crate::frame::Frame;
use crate::radio::Radio;
use crate::streams::{RadioSettings, Rx, Tx};

mod radio;
mod streams;
pub mod frame;
pub mod tools;
pub mod runtime;
pub mod pipeline;

/// This set of bits is sent ahead of the real transmission frame to allow time for the SDR to sync
/// and better be able to read the frame that follows.
pub static AMBLE: &str = "10101010101010101010101010101010";

/// IDENT (AKA identifier) is a set of bits that allows the radio to know that it has reached the
/// end of the preamble and is about to read the frame data.
pub static IDENT: &str = "11110000111100001111000011110000";

/// This is the current Modulation scheme that the radio is configured to accept
pub static MOD_TYPE: ModulationType = ModulationType::BPSK;

/// This is a list of different signal modulation types  currently accepted by the radio
pub enum ModulationType {
    /// Amplitude Shift Keying (ASK) is a very basic modulation type that only allows for 1 bit to
    /// be transmitted at a time. This utilizes the shift in a radio signal's amplitude to encode
    /// binary.
    ///
    /// Advantage:
    /// * Easily can be boosted by use of an amplifier (as amplifiers makes the amplitude bigger)
    ///
    /// Disadvantage:
    /// * Amplitude degrades over distance amplitude sent at TX might be the same height by RX
    ASK,

    /// Frequency Shift Keying (FSK) is a basic modulation type that only allows for 1 bit to
    /// be transmitted at a time. This utilizes the shift in a radio signal's frequency to encode
    /// binary.
    ///
    /// Advantage:
    /// * Very resilient to degrading of a signal over a large distance. If the signal can be
    /// received even if very weak, it will be read.
    ///
    /// Disadvantage:
    /// * If the frequency is occupied by other signals, it becomes very hard to clean and relies on
    /// intensive DSP filtering techniques. Also FSK is believed to put undo strain on a radio
    /// significantly reducing the life span of a radio system,
    FSK,

    /// Binary Phase Shift Keying (BPSK) is a modulation type that only allows for 1 bit to
    /// be transmitted at a time. This utilizes the shift in a radio signal's phase of I component
    /// to encode binary data.
    ///
    /// Advantage:
    /// * This modulation scheme takes 0 bandwidth and is excellent in operating in very noising
    /// environments as it can sit in a sliver of a frequency where no or little transmission takes
    /// place.
    ///
    /// Disadvantage:
    /// * BPSK is very reliant on Phase Locked Loops (PLLs) that ensure the phases are in sync. What
    /// this means if a TX source isn't louder than some other TX device, the an RX device won't be
    /// able to make heads and tails of it.
    BPSK,

    /// Quadrature Phase Shift Keying (QPSK) is a modulation type that only allows for 2 bits to
    /// be transmitted at a time. This utilizes the shift in a radio signal's phase of I and Q 
    /// components to encode binary data.
    ///
    /// Advantage:
    /// * This modulation scheme takes 0 bandwidth and is excellent in operating in very noising
    /// environments as it can sit in a sliver of a frequency where no or little transmission takes
    /// place. It also is able to send 2 bits at once without compromising on reliability.
    ///
    /// Disadvantage:
    /// * QPSK is also very reliant on Phase Locked Loops (PLLs) that ensure the phases are in sync.
    /// What this means if a TX source isn't louder than some other TX device, then an RX device 
    /// won't be able to make heads and tails of it.
    QPSK,
}

/// This will return the number of bits being currently transmitted per a symbol
pub fn bits_per_symbol() -> u8 {
    match MOD_TYPE {
        ModulationType::ASK => { 1 }
        ModulationType::FSK => { 1 }
        ModulationType::BPSK => { 1 }
        ModulationType::QPSK => { 2 }
    }
}

/// This is a helper function that makes it easy to change mod and demod functions on the fly.
pub fn demodulation(obj: &Demodulators, arr: Vec<Complex<f32>>) -> Vec<u8> {
    match MOD_TYPE {
        ModulationType::ASK => { obj.ask(arr) }
        ModulationType::FSK => { obj.fsk(arr) }
        ModulationType::BPSK => { obj.bpsk(arr) }
        ModulationType::QPSK => { obj.qpsk(arr) }
    }
}

/// This is a helper function that makes it easy to change mod and demod functions on the fly.
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


/// Radio stream is the main class of the Radio crate. This makes it easy for anyone to RX or 
/// TX with an SDR from Rust. This is done by providing RX and TX methods through this object
pub struct RadioStream {
    /// The tx stream that data is passed to to have transmitted out of the radio
    pub tx_stream: Tx,

    /// This is a pre-built modulation object that allows for quick modulation of out going data
    pub modulation: Modulators,

    /// This is a thread safe buffer that the RX thread will fill with received transmissions that
    /// can be read from the read method
    pub rx_buffer: Arc<RwLock<Vec<Vec<u8>>>>,

    /// This is a saved copy of the radio settings that can be altered later depending the needs and
    /// wants of the user.
    pub settings: RadioSettings,
}


impl RadioStream {
    /// This will create a new radio stream object.
    ///
    /// This will return an error if no radio is detected or can be connected to.
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
            lpf_filter: 1e5,
            channels_in_use: 0,
            gain: 100.0,
            radio,
            baud_rate: 2e4,
            size: 0,
        };

        // Read buffer
        let buffer = Arc::new(RwLock::new(Vec::with_capacity(20)));

        // Make radio streams
        let me = RadioStream {
            tx_stream: Tx::new(set.clone())?,
            rx_buffer: buffer.clone(),
            settings: set.clone(),
            modulation: Modulators::new((set.sample_rate as f32 / set.baud_rate) as usize, set.sample_rate as f32),
        };


        // Spawn rx thread
        spawn(move || {
            // create stream
            if let Ok(mut rx_stream) = Rx::new(set.clone()) {
                let samples_per_a_symbol = set.sample_rate as f32 / set.baud_rate;

                let mut run = runtime::Runtime::new(samples_per_a_symbol as usize, set.sample_rate as f32, IDENT, buffer);

                // create mtu
                let mut mtu = vec![Complex::new(0.0, 0.0); samples_per_a_symbol as usize];

                // rx loop
                loop {
                    let err = rx_stream.fetch(&[mtu.as_mut_slice()]);

                    if err.is_err() {
                        println!("Error!")
                    }
                    run.run(mtu.clone())
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
        let signal = self.modulation.bpsk(frame.assemble().as_slice());

        // Send
        self.tx_stream.send(signal.as_slice())?;

        Ok(())
    }

    /// This a wrapper function that allows for direct transmission of frames
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

    /// This is a wrapper class that allows for the reading and receiving of frames directly
    pub fn receive_frames(&self) -> Result<Frame> {
        if let Ok(bytes) = self.read() {
            Ok(Frame::from(bytes.as_slice()))
        } else {
            Err(Error::msg("Failed to read from stream!"))
        }
    }
}
