use std::fs;
use std::sync::{Arc, RwLock};

use anyhow::{Error, Result};
use num_complex::Complex;
use soapysdr::{Args, Direction, ErrorCode, RxStream, TxStream};
use serde::{Deserialize, Serialize};

use crate::radio::Radio;

/// settings for configuring a stream
#[derive(Clone, Serialize, Deserialize)]
pub struct RadioSettings {
    /// The rate the radio will take a sample in hz
    pub sample_rate: f64,

    /// The frequency the radio will "down-sample" / "up-sample"
    ///
    /// It should be noted "down-sample" and "up-sample" are NOT the proper terms for what is
    /// happening. The correct terms are down-conversion and up-conversion however, most people
    /// find it easier to understand using the other terms.
    ///
    /// LO stands for local oscillator which is the oscillator some SDRs have that is hooked up to a
    /// circuit called a mixer. This mixer does the black magic of conversion between frequencies.
    /// What this value is doing is setting the frequency of the oscillator connected to the mixer.
    /// For SDRs that don't have LO and are what is called "direct samplers", like LimeSDRs and
    /// BladeRFs, this will still set the conversion values properly.
    ///
    /// # RX Example
    /// `lo_frequency = 144MHz`
    /// `incoming_frequency = 145MHz`
    /// `then`
    /// `new_frequency = 1MHz`
    ///
    /// also
    ///
    /// `lo_frequency = 144MHz`
    /// `incoming_frequency = 143MHz`
    /// `then`
    /// `new_frequency = -1MHz`
    ///
    /// # TX Example
    /// `lo_frequency = 144MHz`
    /// `transmit_frequency = 100KHz`
    /// `then`
    /// `output_frequency = 144.1MHz`
    pub lo_frequency: f64,

    /// The number of Channels the stream is currently using.
    /// The maximum value highly depends on your SDR and how it is configured.
    ///
    /// For LimeSDR-USB: There are 2 channels each having their own "LO". Both channels are
    /// full-duplex.
    ///
    /// For HackRF: There is only one channel and is half-duplex. So, only one channel can be used
    /// at a time
    pub channels_in_use: usize,

    /// Gain of stream
    pub gain: f64,

    /// Amount of bits per a second the radio will read
    pub baud_rate: f32,

    /// Samples per a symbol
    pub sps: usize
}

impl RadioSettings{
    pub fn load_from_file(filename: &str) -> Option<RadioSettings> {
        
        let loaded = fs::read_to_string(filename);
        
        if loaded.is_ok(){
            Some(toml::from_str(loaded.unwrap().as_str()).unwrap())
        }else { 
            None
        }
    }
    
    pub fn save_to_file(&self, filename: &str){
        let output = toml::to_string(self).unwrap();
        fs::write(filename,output).unwrap();
    }
}


/// Rx Stream For Radio
pub struct Rx {
    stream: RxStream<Complex<f32>>,
}

impl Rx {
    pub fn new(radio: Radio, mut settings: RadioSettings) -> Result<Rx, soapysdr::Error> {

        // Get radio
        let device = radio.get_radio()?;

        device.set_bandwidth(Direction::Rx,settings.channels_in_use,40e6).unwrap();

        // Set radio sample rate
        device.set_sample_rate(Direction::Rx, settings.channels_in_use, settings.sample_rate)?;

        // Set gain
        device.set_gain_mode(Direction::Rx, settings.channels_in_use,true)?;

        // Set carrier frequency
        device.set_frequency(Direction::Rx, settings.channels_in_use, settings.lo_frequency, Args::new())?;

        // Get rx stream
        let mut rx = Rx {
            stream: device.rx_stream(&[settings.channels_in_use])?
        };

        // Activate RX stream
        rx.stream.activate(Default::default())?;

        // Increase counter
        settings.channels_in_use += 1;

        Ok(rx)
    }

    pub fn get_mtu(&self) -> usize{
        self.stream.mtu().unwrap()
    }

    /// This function fetches the sample in place (to improve performance)
    pub fn fetch(&mut self, arr: &mut [&mut [Complex<f32>]]) -> Result<()> {
        self.stream.read(arr, 100000000_i64)?;

        Ok(())
    }
}

/// Tx Stream For Radio
#[derive(Clone)]
pub struct Tx {
    stream: Arc<RwLock<TxStream<Complex<f32>>>>,
}

impl Tx {
    pub fn new(radio: Radio, mut settings: RadioSettings) -> Result<Tx, soapysdr::Error> {

        // Get radio
        let device = radio.get_radio()?;

        // Set radio sample rate
        device.set_sample_rate(Direction::Tx, settings.channels_in_use, settings.sample_rate)?;

        device.set_gain(Direction::Tx, settings.channels_in_use, settings.gain)?;

        // Set carrier frequency
        device.set_frequency(Direction::Tx, settings.channels_in_use, settings.lo_frequency, Args::new())?;

        let stream = Arc::new(RwLock::new(device.tx_stream(&[settings.channels_in_use])?));

        // Get rx stream
        let tx = Tx {
            stream: stream.clone()
        };

        let x = if let Ok(mut x) = stream.write() {
            x.activate(Default::default())?;

            // Increase counter
            settings.channels_in_use += 1;

            Ok(tx)
        } else {
            Err(
                soapysdr::Error {
                    code: ErrorCode::StreamError,
                    message: "Unable to start radio! stream!".to_string(),
                }
            )
        };
        x
    }

    pub fn get_mtu(&self) -> usize{
        self.stream.read().unwrap().mtu().unwrap()
    }

    pub fn send(&self, arr: &[Complex<f32>]) -> Result<()> {
        if let Ok(mut x) = self.stream.write() {
            x.write_all(&[arr], Default::default(), true, 100000000_i64)?;

            Ok(())
        } else {
            Err(Error::msg("Unable to send data!".to_string()))
        }
    }
}