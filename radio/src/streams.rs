use num_complex::Complex;
use soapysdr::{Args, Direction, RxStream, TxStream};
use crate::radio::Radio;
use anyhow::Result;

/// settings for configuring a stream
#[derive(Clone)]
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

    /// Low pass filter frequency the radio will filter after lo_frequency down sample
    /// # RX Example
    /// `lpf_filter = 100KHz
    /// incoming_frequency = ±101KHz
    /// then
    /// No Signal Is Received`
    ///
    /// also
    ///
    /// `lpf_filter = 100KHz
    /// incoming_frequency = ±99KHz
    /// then
    /// Signal Is Received`
    pub lpf_filter: f64,

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

    /// Radio to use for stream(s)
    pub radio: Radio,

    /// Amount of bits per a second the radio will read
    pub baud_rate: f64,

    /// Optimized and preferred sample size
    pub size: usize,
}


/// Rx Stream For Radio
pub struct Rx{
    Stream: RxStream<Complex<f32>>,
}

impl Rx {
   pub fn new(mut settings: RadioSettings) -> Result<Rx, soapysdr::Error>{
       // Get radio
       let device = settings.radio.get_radio();

       // Set radio sample rate
       device.set_sample_rate(Direction::Rx, settings.channels_in_use, settings.sample_rate)?;

       // Set gain
       device.set_gain(Direction::Rx,settings.channels_in_use, settings.gain)?;

       // Set carrier frequency
       device.set_frequency(Direction::Rx, settings.channels_in_use, settings.lo_frequency, Args::new())?;

       // Set hardware low pass filter
       //device.set_bandwidth(Direction::Rx, settings.channels_in_use,settings.lpf_filter)?;

       // Get rx stream
       let mut rx = Rx{
           Stream: device.rx_stream(&[settings.channels_in_use])?
       };

       // Activate RX stream
       rx.Stream.activate(Default::default())?;

       settings.size = rx.Stream.mtu()?;

       // Increase counter
       settings.channels_in_use += 1;

       Ok(rx)
   }

    pub fn fetch(&mut self, len:usize) -> Result<Vec<Complex<f32>>>{
        let mut to_return = vec![Complex::new(0.0, 0.0); len];
        // to_return.resize(len,Complex::new(0.0,0.0));

        self.Stream.read(&[to_return.as_mut_slice()], 100000000_i64)?;

        Ok(to_return)
    }
}

/// Tx Stream For Radio
pub struct Tx{
    Stream: TxStream<Complex<f32>>,
}

impl Tx {
    pub fn new(mut settings: RadioSettings) -> Result<Tx, soapysdr::Error>{
        // Get radio
        let device = settings.radio.get_radio();

        // Set radio sample rate
        device.set_sample_rate(Direction::Tx, settings.channels_in_use, settings.sample_rate)?;

        // Set gain
        device.set_gain(Direction::Tx,settings.channels_in_use, settings.gain)?;

        // Set carrier frequency
        device.set_frequency(Direction::Tx, settings.channels_in_use, settings.lo_frequency, Args::new())?;

        // Set hardware low pass filter
        //device.set_bandwidth(Direction::Tx, settings.channels_in_use,settings.lpf_filter)?;

        // Get rx stream
        let mut tx = Tx{
            Stream: device.tx_stream(&[settings.channels_in_use])?
        };

        // Activate RX stream
        tx.Stream.activate(Default::default())?;

        settings.size = tx.Stream.mtu()?;

        // Increase counter
        settings.channels_in_use += 1;

        Ok(tx)
    }

    pub fn send(&mut self, arr: &[Complex<f32>]) -> Result<()>{

        self.Stream.write_all(&[arr],Default::default(), true, 100000000_i64)?;

        Ok(())
    }
}