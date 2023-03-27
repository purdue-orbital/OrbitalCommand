use core::time;
use std::fmt::Error;
use std::thread;
use std::thread::sleep;
use num_complex::Complex;
use crate::stream::{RXStream, StreamSettings, TXStream};
use crate::radio::Radio;


pub struct Pipeline{
    /// Tx Stream
    pub Tx: TXStream,

    /// Rx buffer array
    pub Rx_Buffer: Vec<Complex<f32>>,
}

impl Pipeline {
    /// Create a new radio workflow
    pub fn new(frequency:f64, sample_rate:f64) -> Result<Pipeline, Error>
    {
        // Make a new radio instance
        let mut radio = Radio::new().unwrap();

        // Create Settings
        let mut settings = StreamSettings{
            lo_frequency:frequency,
            lpf_filter: 150e3,
            channel: 0,
            gain: 70.0,
            sample_rate,
            radio,
            listen_time: 0.02,
        };

        // Initialize pipeline
        let mut pipe = Pipeline{
            Tx:TXStream::new(settings.clone())?,
            Rx_Buffer:Vec::new()
        };

        // Start a thread of rapid sampling
        thread::spawn( move ||{

            // Start Rx Stream
            let mut rx = RXStream::new(settings.clone()).unwrap();

            // Keep looping
            loop{
                // Sleep briefly
                sleep(time::Duration::from_secs_f32(0.002));

                // Rx once
                let mut arr = rx.fetch();

                // Spawn processing thread to process rx
                thread::spawn(move || {

                    // TODO: This is where demodulation will happen

                });
            }
        });

        Ok(pipe)
    }

    // TODO: Add a function to retrieve demodulated values

    // TODO: Add a function to send binary values
}