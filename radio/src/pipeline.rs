use crate::radio::Radio;
use crate::stream::{RXStream, StreamSettings, TXStream};
use anyhow::{anyhow, Result};
use core::time;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::TryRecvError;
use num_complex::Complex;
use std::thread;
use std::thread::sleep;
use threadpool::ThreadPool;
use crate::dsp;

pub struct Pipeline {
    /// tx Stream
    tx: TXStream,
    /// Rx buffer array
    rx_buffer: Vec<Complex<f32>>,
    rx_channel: mpsc::Receiver<Vec<u8>>,
}

/// Accumulates binary information and outputs it on a channel once it is complete
struct ByteAccumulator {
    data_len: usize,
    accum: Vec<u8>,
    channel: mpsc::Sender<Vec<u8>>,
    current_byte: u8,
    current_byte_idx: u8,
    current_parity_sum: u8,
}

impl ByteAccumulator {
    fn new(channel: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            data_len: 0,
            accum: vec![],
            channel,
            current_byte: 0,
            current_byte_idx: 0,
            current_parity_sum: 0,
        }
    }

    fn accumulate_bit(&mut self, bit: bool) -> Result<()> {
        // Don't start accumulation until a 1
        if self.data_len == 0 && self.current_byte == 0 && !bit {
            return Ok(());
        }

        // Accumulate against the current bit
        if self.current_byte_idx == 8 {
            self.accumulate_byte(self.current_byte)?;
            self.current_byte_idx = 0;
            self.current_byte = 0;
        }

        self.current_byte |= (bit as u8) << self.current_byte_idx;
        self.current_byte_idx += 1;

        Ok(())
    }

    fn accumulate_byte(&mut self, byte: u8) -> Result<()> {
        // If there's no data length configured, configure it now
        if self.data_len == 0 {
            self.data_len = (byte >> 1) as usize;
            return Ok(());
        }

        // If there have been 7 bytes pushed, this next byte MUST be a parity check byte
        if (self.accum.len() + 1) % 8 == 0 {
            if byte != 0b10101000 | self.current_parity_sum {
                self.current_parity_sum = 0;

                self.accum.clear();
                self.data_len = 0;
                return Ok(());
            }
            self.current_parity_sum = 0;
        } else {
            let mut ones = 0;
            for i in 0..8 {
                if byte >> i & 1 == 1 {
                    ones += 1;
                }
            }

            if ones % 2 == 1 {
                self.current_parity_sum += 1;
            }
        }

        self.accum.push(byte);

        if self.accum.len() == self.data_len {
            // Find all indices such that (idx + 1) % 8 == 0 and remove them last to first to clear parity data
            for i in (7..self.accum.len()).step_by(8).rev() {
                self.accum.remove(i);
            }
            self.channel.send(self.accum.clone())?;
            self.accum.clear();
            self.data_len = 0;
        }

        Ok(())
    }
}

impl Pipeline {
    /// Create a new radio workflow
    pub fn new(frequency: f64, sample_rate: f64) -> Result<Pipeline> {
        // Make a new radio instance
        let mut radio = Radio::new()?;
        let (tx, rx) = mpsc::channel();

        let pool = ThreadPool::new(100);

        // Create Settings
        let mut settings = StreamSettings {
            lo_frequency: frequency,
            lpf_filter: 150e3,
            channel: 0,
            gain: 70.0,
            sample_rate,
            radio,
            listen_time: 0.2,
        };

        // Initialize pipeline
        let mut pipe = Pipeline {
            tx: TXStream::new(settings.clone())?,
            rx_buffer: Vec::new(),
            rx_channel: rx,
        };

        // Start a thread of rapid sampling
        thread::spawn(move || {
            // Start Rx Stream
            let mut rx = RXStream::new(settings.clone()).unwrap();

            let accumulator = Arc::new(Mutex::new(ByteAccumulator::new(tx)));
            // Keep looping
            loop {
                // Sleep briefly
                sleep(time::Duration::from_secs_f32(0.002));

                // Rx once
                let mut arr = rx.fetch();

                // Spawn processing thread to process rx

                let accumulator = accumulator.clone();
                pool.execute(move || {
                    let mut demoded = dsp::Demodulators::fsk(arr, sample_rate, 0.0001);
                    {
                        let mut locked = accumulator.lock().unwrap();
                        for char in demoded.chars() {
                            locked.accumulate_bit(char == '1').unwrap();
                        }
                    }

                    // TODO: This is where demodulation will happen
                });
            }
        });

        Ok(pipe)
    }

    pub fn recv(&self) -> Result<Option<Vec<u8>>> {
        match self.rx_channel.try_recv() {
            Ok(vec) => Ok(Some(vec)),
            Err(e) => match e {
                TryRecvError::Empty => Ok(None),
                TryRecvError::Disconnected => Err(anyhow!("Channel disconnected!")),
            }
        }
    }

    // TODO: Add a function to send binary values
    pub fn send(&self, bytes: &[u8]) -> Result<()> {
        // TODO: Inject parity data
        Ok(())
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        todo!()
    }
}
