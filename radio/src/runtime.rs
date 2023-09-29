//! Runtime strictly handles the RX runtime and each step in handling a new set of samples

use std::sync::{Arc, RwLock};

use num_complex::Complex;

use bitvec::prelude::*;
use bytes::Bytes;
use flume::{Receiver, Sender};
use rustdsp::Demodulators;
use rustdsp::filters::fir;
use rustdsp::filters::fir::shapes::WindowShapes::Rectangle;

use crate::frame::{Frame, IDENT_VEC};
use crate::pipeline::{create_bytes_channel, middle_man};
use crate::pipeline::frame::{decode_task, encode_task};
use crate::pipeline::ident_search::search_task;

fn shift_and_carry(bin: &mut [u8], bit: u8) {
    let view = bin.view_bits_mut::<Msb0>();
    view.shift_left(1);

    bin[bin.len() - 1] |= bit;
}

pub struct Runtime {
    current_samples: Vec<Complex<f32>>,

    filter_instance: fir::Windowing,

    demod_instance: Demodulators,

    demoded_value: u8,

    ident_window: Vec<u8>,

    record_window: Vec<u8>,

    bin: u8,

    bin_counter: u8,

    state_counter: u8,

    buffer: Arc<RwLock<Vec<Vec<u8>>>>,

    start: Sender<u8>,

    end: Receiver<Bytes>,
}

impl Runtime {
    /// Generate a new runtime instance
    pub fn new(samples_per_symbol: usize, sample_rate: f32, ident: &str, buffer: Arc<RwLock<Vec<Vec<u8>>>>) -> Runtime {

        let (tx_start, rx_start) = flume::unbounded();
        let (searcher, rx_search) = search_task::Task::new(rx_start);
        let (decoder, rx_decode) = decode_task::Task::new(rx_search);

        decoder.start();
        searcher.start();

        Runtime {
            current_samples: vec![],

            filter_instance: fir::Windowing::new(Rectangle, samples_per_symbol, 0),

            demod_instance: Demodulators::new(samples_per_symbol, sample_rate),

            demoded_value: 0,

            ident_window: vec![0; 3*(IDENT_VEC.len())],

            record_window: vec![],

            bin: 0,

            bin_counter: 0,

            state_counter: 0,

            buffer,

            start: tx_start,

            end: rx_decode,
        }
    }

    /// What to run on demod state
    pub fn demod(&mut self) {
        unsafe {self.demoded_value = self.demod_instance.bpsk(self.current_samples.clone())[0]};
    }

    pub fn eval(&mut self){
        self.bin <<= 1;
        self.bin |= self.demoded_value;
        self.bin_counter += 1;

        if self.bin_counter == 8{
            unsafe {self.start.send(self.bin).unwrap_unchecked()};
            self.bin_counter = 0;
            self.bin = 0;
        }
    }

    /// Run runtime
    pub fn run(&mut self, samples: Vec<Complex<f32>>) {
        // set samples
        self.current_samples = samples;

        // filter
        //self.filter();

        // demod
        self.demod();

        self.eval();

       if let Ok(x) = self.end.try_recv(){
           unsafe {self.buffer.write().unwrap_unchecked().push(x.to_vec())}
       }
    }
}