use std::fmt::Error;
use std::ops::DerefMut;
use std::sync::mpsc::channel;
use num_complex::Complex;
use soapysdr::{Args, Direction, RxStream, TxStream};
use crate::radio::Radio;

/// Settings for configuring a stream
#[derive(Clone)]
pub struct StreamSettings{
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

    /// Channel the stream is going to operate on starting at index 0.
    /// This value highly depends on your SDR and how it is configured.
    ///
    /// For LimeSDR-USB: There are 2 channels each having their own "LO". Both channels are
    /// full-duplex.
    ///
    /// For HackRF: There is only one channel and is half-duplex. So, by default, this value will be
    /// 0.
    pub channel: usize,

    /// Gain of stream
    pub gain: f64,

    /// Radio to use for stream(s)
    pub radio: Radio,

    /// Amount of time radio should listen for in seconds per a RX call (RX ONLY)
    pub listen_time: f64,
}


/// This is a stream for fetching radio transmissions and only fetching
pub struct RXStream {
    Settings: StreamSettings,
    Stream: RxStream<Complex<f32>>,
    Size: usize,
}

/// This is a stream for sending radio transmissions and only sending
pub struct TXStream {
    Settings: StreamSettings,
    Stream: TxStream<Complex<f32>>,
}

/// This is a stream for fetching radio transmissions and only fetching
impl RXStream {
    pub fn new(settings:StreamSettings) -> Result<RXStream,Error>{

        // Set radio center frequency
        settings.radio.get_radio().set_frequency(
            Direction::Rx,
            settings.channel,
            settings.lo_frequency,
            Args::new()
        ).unwrap();

        // Set radio sample rate
        settings.radio.get_radio().set_sample_rate(
            Direction::Rx,
            settings.channel,
            settings.sample_rate
        ).unwrap();

        // Set radio bandwidth
        settings.radio.get_radio().set_bandwidth(
            Direction::Rx,
            settings.channel,
            settings.lpf_filter
        ).unwrap();

        // Set gain for stream
        settings.radio.get_radio().set_gain(
            Direction::Rx,
            settings.channel,
            settings.gain
        ).unwrap();

        // Initialize stream
        let mut stream = RXStream {
            Settings: settings.clone(),
            Stream: settings.radio.get_radio().rx_stream(&[settings.channel]).unwrap(),
            Size: (settings.sample_rate * settings.listen_time) as usize,
        };

        stream.Stream.activate(None).unwrap();

        Ok(stream)
    }

    /// Get RX data
    pub fn fetch(&mut self) -> Vec<Complex<f32>> {

        // Make array
        let mut arr = Vec::new();
        arr.resize(self.Size,Complex::new(0.0,0.0));

        // Fill array
        self.Stream.read(&[arr.as_mut_slice()],0).unwrap();

        // Return the now full array
        arr.to_vec()
    }

}

/// This is a stream for sending radio transmissions and only sending
impl TXStream{
    pub fn new(settings: StreamSettings) -> Result<TXStream,Error>{

        // Set radio center frequency
        settings.radio.get_radio().set_frequency(
            Direction::Tx,
            settings.channel,
            settings.lo_frequency,
            Args::new()
        ).unwrap();

        // Set radio sample rate
        settings.radio.get_radio().set_sample_rate(
            Direction::Tx,
            settings.channel,
            settings.sample_rate
        ).unwrap();

        // Set gain for stream
        settings.radio.get_radio().set_gain(
            Direction::Tx,
            settings.channel,
            settings.gain
        ).unwrap();

        let mut stream = TXStream {
            Settings: settings.clone(),
            Stream: settings.radio.get_radio().tx_stream(&[settings.channel]).unwrap(),
        };

        stream.Stream.activate(None).unwrap();

        Ok(stream)

    }

    /// Transmit modulated radio data.
    /// Passing Vec<Complex<f32>> will have it transmitted through the given stream settings
    pub fn transmit(&mut self, arr : Vec<Complex<f32>>){
        self.Stream.write_all(&[arr.as_slice()], None, false, 0).unwrap();
    }
}
