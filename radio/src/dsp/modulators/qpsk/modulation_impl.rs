use std::f32::consts::PI;

use num_complex::Complex;

use crate::dsp::modulators::qpsk::structs::modulation::Modulation;
use crate::dsp::common::generate_wave::generate_wave;

impl Modulation {
    pub fn new(samples_per_symbol: usize, sample_rate: f32, message_signal: f32) -> Modulation {
        Modulation {
            samples_per_symbol,
            sample_rate,

            signal_0: generate_wave(message_signal, sample_rate, samples_per_symbol as i32, 0, 1.0, PI, PI),
            signal_1: generate_wave(message_signal, sample_rate, samples_per_symbol as i32, 0, 1.0, PI, 0.0),
            signal_2: generate_wave(message_signal, sample_rate, samples_per_symbol as i32, 0, 1.0, 0.0, PI),
            signal_3: generate_wave(message_signal, sample_rate, samples_per_symbol as i32, 0, 1.0, 0.0, 0.0),
        }
    }

    /// Modulate a radio signal using qpsk
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn run(&self, bin: &[u8]) -> Vec<Complex<f32>> {
        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * self.samples_per_symbol);

        for &x in bin {
            for y in (0..8).step_by(2) {
                let val = (x << y) >> 6;

                to_return.extend(
                    match val {
                        1 => { self.signal_1.as_slice() }
                        2 => { self.signal_2.as_slice() }
                        3 => { self.signal_3.as_slice() }

                        // defualt as 0
                        _ => { self.signal_0.as_slice() }
                    }
                )
            }
        }


        to_return
    }
}