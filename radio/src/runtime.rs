//! Runtime strictly handles the RX runtime and each step in handling a new set of samples

use std::sync::{Arc, RwLock};

use num_complex::Complex;

use dsp::filters::fir;

use crate::dsp;
use crate::dsp::Demodulators;
use crate::dsp::filters::fir::shapes::WindowShapes::Rectangle;
use crate::frame::{Frame, IDENT_VEC};

fn shift_and_carry(bin: &mut [u8], bit: u8) {

    // set carry bit
    let mut carry = bit & 1;

    // shift then add carry
    for x in bin.iter_mut().rev() {
        // save new carry bit
        let new_carry_bit = (*x >> 7) & 1;

        // shift and add carry bit
        *x = (*x << 1) + carry;

        // add new carry bit
        carry = new_carry_bit;
    }
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
}

impl Runtime {
    /// Generate a new runtime instance
    pub fn new(samples_per_symbol: usize, sample_rate: f32, ident: &str, buffer: Arc<RwLock<Vec<Vec<u8>>>>) -> Runtime {
        Runtime {
            current_samples: vec![],

            filter_instance: fir::Windowing::new(Rectangle, samples_per_symbol, 0),

            demod_instance: Demodulators::new(samples_per_symbol, sample_rate),

            demoded_value: 0,

            ident_window: vec![0; 2 * (IDENT_VEC.len())],

            record_window: vec![],

            bin: 0,

            bin_counter: 0,

            state_counter: 0,

            buffer,
        }
    }

    /// What to run on filter state
    pub fn filter(&mut self) {
        self.filter_instance.run(&mut self.current_samples);
    }

    /// What to run on demod state
    pub fn demod(&mut self) {
        self.demoded_value = self.demod_instance.bpsk(self.current_samples.clone())[0];
    }

    /// What to run on evaluate state
    pub fn evaluate(&mut self) {
        match self.state_counter {
            // listen state
            0 => {
                shift_and_carry(self.ident_window.as_mut_slice(), self.demoded_value);

                let test_frame = Frame::from(self.ident_window.as_slice());

                if test_frame.has_ident {
                    self.state_counter = 1;

                    self.record_window = self.ident_window.clone();
                }
            }

            // record state
            1 => {
                self.bin = (self.bin << 1) ^ self.demoded_value;
                self.bin_counter += 1;

                // if we fill the bin up, add to record window and then re-evaluate if at end of transmission
                if self.bin_counter == 8 {
                    self.record_window.push(self.bin);

                    self.bin_counter = 0;
                    self.bin = 0;

                    let test_frame = Frame::from(self.record_window.as_slice());

                    if test_frame.is_complete {
                        self.state_counter = 0;
                        unsafe { self.buffer.write().unwrap_unchecked().push(test_frame.data) }
                    }
                }
            }

            _ => {}
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

        // evaluate
        self.evaluate();
    }
}