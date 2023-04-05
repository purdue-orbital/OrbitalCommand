use std::thread;
use std::time::Duration;
use crate::pipeline::Pipeline;

use reed_solomon::Encoder;
use reed_solomon::Decoder;

use rand::distributions::{Distribution};
use rand_distr::{Normal};
use rand::Rng;

use num_complex::ComplexFloat;

mod pipeline;
mod stream;
mod radio;
mod dsp;
mod tools;

use num_complex::Complex;

const FREQ_0: f32 = 1000.0;
const FREQ_1: f32 = 2000.0;
const SYMBOL_RATE: f32 = 100.0;
const MOD_INDEX: f32 = 0.5;
const SAMPLE_RATE: f32 = 44100.0;
const SYMBOL_LENGTH: f32 = 1.0 / SYMBOL_RATE;
const SAMPLE_LENGTH: f32 = 1.0 / SAMPLE_RATE;
const DEMOD_THRESHOLD: f32 = 1.002;

const PHASE_INCREMENTS: [[f32; 2]; 2] = [
    [
        2.0 * std::f32::consts::PI * FREQ_0 * SAMPLE_LENGTH,
        2.0 * std::f32::consts::PI * (FREQ_0 + MOD_INDEX * FREQ_0 * SYMBOL_LENGTH) * SAMPLE_LENGTH,
    ],
    [
        2.0 * std::f32::consts::PI * FREQ_1 * SAMPLE_LENGTH,
        2.0 * std::f32::consts::PI * (FREQ_1 + MOD_INDEX * FREQ_1 * SYMBOL_LENGTH) * SAMPLE_LENGTH,
    ],
];

fn fsk_modulate(bits: &[u8]) -> Vec<Complex<f32>> {
    let n = (SYMBOL_LENGTH / SAMPLE_LENGTH) as usize;
    bits.iter().flat_map(|b| {
        let [p0, p1] = PHASE_INCREMENTS[*b as usize];
        (0..n).scan(0.0, move |phase, _| {
            let s = Complex::<f32>::from_polar(1.0, *phase);
            *phase += if *phase < std::f32::consts::PI { p0 } else { p1 };
            Some(s)
        })
    }).collect()
}

fn fsk_demodulate(signal: &[Complex<f32>]) -> Vec<u8> {
    let n = (SYMBOL_LENGTH / SAMPLE_LENGTH) as usize;
    signal.chunks(n).map(|chunk| {
        let (c0, c1) = chunk.iter().map(|c| (c.arg(), c.norm())).unzip::<_, _, Vec<_>, Vec<_>>();
        let diff = (c1.iter().sum::<f32>() / c1.len() as f32) - (c0.iter().sum::<f32>() / c0.len() as f32);
        println!("{}",diff);
        if diff > DEMOD_THRESHOLD { 1 } else { 0 }
    }).collect()
}

fn add_noise(signal: &[Complex<f32>], snr_db: f32) -> Vec<Complex<f32>> {
    let snr = 10.0f32.powf(snr_db / 10.0); // calculate signal-to-noise ratio
    let signal_power = signal.iter().map(|x| x.norm_sqr()).sum::<f32>() / signal.len() as f32;
    let noise_power = signal_power / snr;
    let standard_deviation = (noise_power / 2.0).sqrt();

    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, standard_deviation);

    signal.iter()
        .map(|&x| {
            let real = normal.unwrap().sample(&mut rng);
            let imag = normal.unwrap().sample(&mut rng);
            x + Complex::new(real, imag)
        })
        .collect()
}


fn main() {

    let mut bin = [0,1,0];

    let mut hold = fsk_modulate(&bin);


    let mut noisy = add_noise(&*hold, 2 as f32);

    dsp::Graph::time_graph("data1.png",noisy.clone());

    let mut arr = fsk_demodulate(&noisy);

    for x in arr{
        println!("{}", x)
    }
}
