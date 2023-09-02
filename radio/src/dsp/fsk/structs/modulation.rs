use num_complex::Complex;

pub struct Modulation {
    // Calculate the number of samples per a symbol
    pub(crate) samples_per_symbol: usize,

    // The rate the radio will sample at
    pub(crate) sample_rate: f32,

    // Pre generated FSK signals
    pub(crate) fsk_one_signal: Vec<Complex<f32>>,
    pub(crate) fsk_zero_signal: Vec<Complex<f32>>,
}