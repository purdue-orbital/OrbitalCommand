use std::f32::consts::PI;
use num_complex::Complex;
use crate::dsp::qpsk::structs::modulation::Modulation;
use crate::dsp::tools::generate_wave::generate_wave;

static QPSK_FREQUENCY: f32 = 100.0;

impl Modulation {

    pub fn new(samples_per_symbol: usize, sample_rate:f32)->Modulation{
        Modulation{samples_per_symbol, sample_rate}
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

                let val= (x << y) >> 6;

                to_return.extend(
                    match val {
                        1 => { generate_wave(QPSK_FREQUENCY, self.sample_rate, self.samples_per_symbol as i32, 0, 1.0, PI, 0.0) }
                        2 => { generate_wave(QPSK_FREQUENCY, self.sample_rate, self.samples_per_symbol as i32, 0, 1.0, 0.0, PI) }
                        3 => { generate_wave(QPSK_FREQUENCY, self.sample_rate, self.samples_per_symbol as i32, 0, 1.0, 0.0, 0.0) }

                        // defualt as 0
                        _ => { generate_wave(QPSK_FREQUENCY, self.sample_rate, self.samples_per_symbol as i32, 0, 1.0, PI, PI) }
                    }
                )
            }
        }


        to_return
    }
}