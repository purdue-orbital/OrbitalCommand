use num_complex::Complex;

pub struct Modulation {
    // Calculate the number of samples per a symbol
    pub(crate) samples_per_symbol: usize,

    // The rate the radio will sample at
    pub(crate) sample_rate: f32,

    pub(crate) signal_0: Vec<Complex<f32>>,
    pub(crate) signal_1: Vec<Complex<f32>>,
    pub(crate) signal_2: Vec<Complex<f32>>,
    pub(crate) signal_3: Vec<Complex<f32>>,
}