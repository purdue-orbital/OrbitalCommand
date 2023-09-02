use std::f32::consts::PI;
use num_complex::Complex;
use crate::dsp::fsk::structs::modulation::Modulation;
use crate::dsp::tools::bi_signal_generation::bi_signal_modulation;
use crate::dsp::tools::generate_wave::generate_wave;

pub static FSK_FREQUENCY1: f32 = 1.0;

pub static FSK_FREQUENCY2: f32 = 1e4;

static BPSK_FREQUENCY: f32 = 100.0;

impl Modulation {
    
    pub fn new(samples_per_symbol: usize, sample_rate:f32)->Modulation{


        let fsk_one_signal = generate_wave(FSK_FREQUENCY2, sample_rate, samples_per_symbol as i32, 0, 1.0,0.0,0.0);
        let fsk_zero_signal = generate_wave(FSK_FREQUENCY1, sample_rate, samples_per_symbol as i32, 0, 1.0,0.0,0.0);

        Modulation{samples_per_symbol,sample_rate,fsk_one_signal,fsk_zero_signal}
    }

    /// Modulate a radio signal using fsk
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn run(&self, bin: &[u8]) -> Vec<Complex<f32>> {
        bi_signal_modulation(bin, &self.fsk_zero_signal, &self.fsk_one_signal, self.samples_per_symbol)
    }
    

}