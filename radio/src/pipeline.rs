use crate::radio::Radio;
use crate::stream::{RXStream, StreamSettings, TXStream};
use anyhow::Result;
use core::time;
use num_complex::Complex;
use std::thread;
use std::thread::sleep;
use threadpool::ThreadPool;

pub struct Pipeline {
    /// tx Stream
    pub tx: TXStream,
    /// Rx buffer array
    pub rx_buffer: Vec<Complex<f32>>,
}

impl Pipeline {
    /// Create a new radio workflow
    pub fn new(frequency: f64, sample_rate: f64) -> Result<Pipeline> {
        // Make a new radio instance
        let mut radio = Radio::new()?;

        let pool = ThreadPool::new(10);

        // Create Settings
        let mut settings = StreamSettings {
            lo_frequency: frequency,
            lpf_filter: 150e3,
            channel: 0,
            gain: 70.0,
            sample_rate,
            radio,
            listen_time: 0.02,
        };

        // Initialize pipeline
        let mut pipe = Pipeline {
            tx: TXStream::new(settings.clone())?,
            rx_buffer: Vec::new(),
        };

        // Start a thread of rapid sampling
        thread::spawn(move || {
            // Start Rx Stream
            let mut rx = RXStream::new(settings.clone()).unwrap();

            // Keep looping
            loop {
                // Sleep briefly
                sleep(time::Duration::from_secs_f32(0.002));

                // Rx once
                let mut arr = rx.fetch();

                // Spawn processing thread to process rx
                pool.execute(move || {

                    // TODO: This is where demodulation will happen
                });
            }
        });

        Ok(pipe)
    }

    // TODO: Add a function to retrieve demodulated values

    // TODO: Add a function to send binary values
}
