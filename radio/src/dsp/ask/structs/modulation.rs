use num_complex::Complex;

pub struct Modulation {
    // Calculate the number of samples per a symbol
    pub(crate) samples_per_symbol: usize,

    // The rate the radio will sample at
    pub(crate) sample_rate: f32,

    // ask pre-generated signals
    pub(crate) ask_on_signal: Vec<Complex<f32>>,
    pub(crate) ask_off_signal: Vec<Complex<f32>>,
}