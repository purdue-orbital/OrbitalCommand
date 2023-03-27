use std::hash::Hash;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;
use std::{process, thread};
use std::fmt::Debug;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use log::error;
use threadpool::ThreadPool;
use crate::dsp::{amplitude_array, generate_wave};
use crate::stream::{RxStream, TxStream};
use crate::tools::{average_array, sum};

pub mod dsp;
pub mod graphy;
pub mod radio;
pub mod stream;
mod tools;

#[derive(PartialEq)]
pub struct FrequencyRange {
    pub center_frequency: f64,
    pub lpf_bandwidth: f64,
}

// This function can be used for help setting explicit ranges of frequencies you want the radio to listen on
pub fn frequency_range(start_frequency: f64, stop_frequency: f64) -> FrequencyRange {
    FrequencyRange {
        center_frequency: (start_frequency + stop_frequency) / 2.0,
        lpf_bandwidth: (stop_frequency - start_frequency),
    }
}

/// Accumulates binary information and outputs it on a channel once it is complete
struct ByteAccumulator {
    data_len: usize,
    accum: Vec<u8>,
    channel: mpsc::Sender<Vec<u8>>,
    current_byte: u8,
    current_byte_idx: u8,
}

impl ByteAccumulator {
    fn new(channel: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            data_len: 0,
            accum: vec![],
            channel,
            current_byte: 0,
            current_byte_idx: 0,
        }
    }

    fn accumulate_bit(&mut self, bit: bool) -> anyhow::Result<()> {
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

    fn accumulate_byte(&mut self, byte: u8) -> anyhow::Result<()> {
        // If there's no data length configured, configure it now
        if self.data_len == 0 {
            self.data_len = (byte >> 1) as usize;
            return Ok(());
        }

        self.accum.push(byte);

        if self.accum.len() == self.data_len {
            self.channel.send(self.accum.clone())?;
            self.accum.clear();
            self.data_len = 0;
        }

        Ok(())
    }
}

// 2^7 - 1
const MAX_BYTES: usize = 127;
const THREAD_SLEEP_MILLIS: u64 = 50;
const PULSE_SLEEP_MICROS: u64 = 900;
const READER_WORKERS: usize = 10;

pub struct RadioReader {
    run: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    channel: mpsc::Receiver<Vec<u8>>,
}

impl RadioReader {
    pub fn new(mut stream: RxStream) -> Self {

        let run = Arc::new(AtomicBool::new(true));
        let (tx, channel) = mpsc::channel::<Vec<u8>>();

        let run_thread = run.clone();
        let pool = ThreadPool::new(READER_WORKERS);
        let handle = thread::spawn(move || for id in 0..READER_WORKERS { {

            // Get last set of data
            for _ in 0..100 {
                stream.rx();
            }

            let mut arr = stream.rx();

            stream.clear_buffer();

            let mut accum = ByteAccumulator::new(tx.clone());
            pool.execute(move || {

                // Make radio wave into amplitude array
                let mut amp_arr = amplitude_array(arr);

                // Average the amplitude values
                let mut amp_avg = average_array(amp_arr,1000,Some(300.0));

                // Find the Average of the Averages
                let amp_avg_avg = sum(amp_avg.clone(),None) / amp_avg.len() as f32;

                // Reduce all values by the average.
                for x in 0..amp_avg.len() {

                    // Reduce value by average
                    let mut i = (*amp_avg.get(x).unwrap()) - amp_avg_avg;

                    // If value is negative, make it zero
                    i *= (i > 0.0) as i32 as f32;

                    amp_avg[x] = i;
                }

                let mut counter = 0;
                let mut last_counter = 0;
                let mut bin = "".to_owned();

                // Modulate Values
                while counter < amp_avg.len() {
                    if amp_avg[counter] > 0.01 {
                        if counter - last_counter > 10 {
                            let mut hold = (counter - last_counter) as i32;

                            hold -= 3300;

                            while hold > 0 {
                                accum.accumulate_bit(false).unwrap();
                                bin.push('0');
                                hold -= 3300;
                            }

                            accum.accumulate_bit(true).unwrap();
                            bin.push('1');
                        }
                        last_counter = counter;
                    }

                    counter += 1;
                }

                if(bin.len() > 0)
                {
                    graphy::graph_vec(("data".to_owned()+id.to_string().as_str()+".png").as_str(),amp_avg);
                }

            });
        }});

        Self {
            run,
            handle: Some(handle),
            channel,
        }
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        match self.channel.try_recv() {
            Ok(vec) => Some(vec),
            Err(e) => match e {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => panic!("Receive channel disconnected!"),
            }
        }
    }
}

impl Drop for RadioReader {
    fn drop(&mut self) {
        self.run.store(false, Ordering::SeqCst);
        self.handle.take().unwrap().join().unwrap();
    }
}

pub struct RadioWriter {
    handle: Option<JoinHandle<()>>,
    /// Ensures that the sender is only ever accessed from one thread at a time
    channel: Mutex<mpsc::Sender<u8>>,
}

impl RadioWriter {
    pub fn new(mut stream: TxStream, frequency: f64, sample_rate: f64, num_samples: i32) -> Self {
        let (channel, rx) = mpsc::channel();
        let wave = generate_wave(frequency, sample_rate, num_samples);
        let handle = thread::spawn(move || loop {
            match rx.try_recv() {
                Ok(val) => {
                    for shift in 0..8 {
                        if val >> shift & 1u8 == 1u8 {
                            if let Err(e) = stream.tx(wave.as_slice()) {
                                error!("Unable to write bit! {e}");
                            }
                        }

                        sleep(Duration::from_micros(PULSE_SLEEP_MICROS));
                    }
                }
                Err(e) => match e {
                    TryRecvError::Empty => sleep(Duration::from_millis(THREAD_SLEEP_MILLIS)),
                    TryRecvError::Disconnected => return,
                }
            }
        });

        Self {
            handle: Some(handle),
            channel: Mutex::new(channel),
        }
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        assert!(data.len() <= MAX_BYTES);

        // Starting one bit then length of data stream
        let locked_channel = self.channel.lock().unwrap();
        locked_channel.send((data.len() as u8) << 1 | 1)?;

        for byte in data {
            locked_channel.send(*byte)?;
        }

        Ok(())
    }
}

impl Drop for RadioWriter {
    fn drop(&mut self) {
        // Forcefully drop the channel to allow the thread to join
        drop(self.channel.lock().unwrap());
        self.handle.take().unwrap().join().unwrap();
    }
}