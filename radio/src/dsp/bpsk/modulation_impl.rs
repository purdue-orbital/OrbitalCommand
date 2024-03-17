use std::f32::consts::PI;

use num_complex::Complex;

use crate::dsp::bpsk::structs::modulation::Modulation;
use crate::dsp::tools::bi_signal_generation::bi_signal_modulation;
use crate::dsp::tools::generate_wave::generate_wave;

static BPSK_FREQUENCY: f32 = 100.0;

impl Modulation {
    pub fn new(samples_per_symbol: usize, sample_rate: f32) -> Modulation {
        let bpsk_zero_signal = generate_wave(
            BPSK_FREQUENCY,
            sample_rate,
            samples_per_symbol as i32,
            0,
            1.0,
            PI,
            0.0,
        );
        let bpsk_one_signal = generate_wave(
            BPSK_FREQUENCY,
            sample_rate,
            samples_per_symbol as i32,
            0,
            1.0,
            0.0,
            0.0,
        );

        Modulation {
            samples_per_symbol,
            sample_rate,
            bpsk_one_signal,
            bpsk_zero_signal,
        }
    }

    /// Modulate a radio signal using bpsk
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn run(&self, bin: &[u8]) -> Vec<Complex<f32>> {
        bi_signal_modulation(
            bin,
            &self.bpsk_zero_signal,
            &self.bpsk_one_signal,
            self.samples_per_symbol,
        )
    }
}
