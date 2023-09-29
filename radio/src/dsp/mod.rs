//! Digital Signal Processing (DSP) is the hallmark and a widely studied field in radio
//! communications and is 

use num_complex::Complex;

use ask::structs::demodulation::Demodulation as ask_demod;
use ask::structs::modulation::Modulation as ask_mod;
use bpsk::structs::demodulation::Demodulation as bpsk_demod;
use bpsk::structs::modulation::Modulation as bpsk_mod;
use fsk::structs::demodulation::Demodulation as fsk_demod;
use fsk::structs::modulation::Modulation as fsk_mod;
use qpsk::structs::demodulation::Demodulation as qpsk_demod;
use qpsk::structs::modulation::Modulation as qpsk_mod;

pub mod tools;
pub mod qpsk;
pub mod bpsk;
pub mod ask;
pub mod fsk;
pub mod filters;
pub mod viterbi;
pub mod wtf_ecc;
pub mod digital_pipeline;


pub struct Demodulators {
    ask: ask_demod,
    fsk: fsk_demod,
    bpsk: bpsk_demod,
    qpsk: qpsk_demod,
}

pub struct Modulators {
    ask: ask_mod,
    fsk: fsk_mod,
    bpsk: bpsk_mod,
    qpsk: qpsk_mod,
}

impl Demodulators {
    pub fn new(samples_per_symbol: usize, sample_rate: f32) -> Demodulators {
        let mut to_return = Demodulators {
            ask: ask_demod::new(0, 0.0, 0.0),
            fsk: fsk_demod::new(0, 0.0, 0.0),
            bpsk: bpsk_demod::new(0, 0.0),
            qpsk: qpsk_demod::new(0, 0.0),
        };

        to_return.update(samples_per_symbol, sample_rate);

        to_return
    }

    pub fn update(&mut self, samples_per_symbol: usize, sample_rate: f32) {
        let message_signal = samples_per_symbol as f32;

        self.ask = ask_demod::new(samples_per_symbol, sample_rate, message_signal);
        self.fsk = fsk_demod::new(samples_per_symbol, sample_rate, sample_rate / 2.0);
        self.bpsk = bpsk_demod::new(samples_per_symbol, sample_rate);
        self.qpsk = qpsk_demod::new(samples_per_symbol, sample_rate);
    }

    pub fn ask(&self, arr: Vec<Complex<f32>>) -> Vec<u8> {
        self.ask.run(arr)
    }
    pub fn fsk(&self, arr: Vec<Complex<f32>>) -> Vec<u8> {
        self.fsk.run(arr)
    }
    pub fn bpsk(&self, arr: Vec<Complex<f32>>) -> Vec<u8> {
        self.bpsk.run(arr)
    }
    pub fn qpsk(&self, arr: Vec<Complex<f32>>) -> Vec<u8> {
        self.qpsk.run(arr)
    }
}

impl Modulators {
    pub fn new(samples_per_symbol: usize, sample_rate: f32) -> Modulators {
        let mut to_return = Modulators {
            ask: ask_mod::new(0, 0.0, 0.0),
            fsk: fsk_mod::new(0, 0.0, 0.0, 0.0),
            bpsk: bpsk_mod::new(0, 0.0, 0.0),
            qpsk: qpsk_mod::new(0, 0.0, 0.0),
        };

        to_return.update(samples_per_symbol, sample_rate);

        to_return
    }


    pub fn update(&mut self, samples_per_symbol: usize, sample_rate: f32) {
        let message_signal = samples_per_symbol as f32;

        self.ask = ask_mod::new(samples_per_symbol, sample_rate, message_signal);
        self.fsk = fsk_mod::new(samples_per_symbol, sample_rate, message_signal, sample_rate / 2.0);
        self.bpsk = bpsk_mod::new(samples_per_symbol, sample_rate, message_signal);
        self.qpsk = qpsk_mod::new(samples_per_symbol, sample_rate, message_signal);
    }

    pub fn ask(&self, arr: &[u8]) -> Vec<Complex<f32>> {
        self.ask.run(arr)
    }
    pub fn fsk(&self, arr: &[u8]) -> Vec<Complex<f32>> {
        self.fsk.run(arr)
    }
    pub fn bpsk(&self, arr: &[u8]) -> Vec<Complex<f32>> {
        self.bpsk.run(arr)
    }
    pub fn qpsk(&self, arr: &[u8]) -> Vec<Complex<f32>> {
        self.qpsk.run(arr)
    }
}
