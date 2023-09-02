use num_complex::Complex;

pub struct Modulation{
    // Calculate the number of samples per a symbol
    pub(crate) samples_per_symbol: usize,

    // The rate the radio will sample at
    pub(crate) sample_rate: f32,
    
    pub(crate) mfsk_freq_map: Vec<Vec<Complex<f32>>>,
    
}