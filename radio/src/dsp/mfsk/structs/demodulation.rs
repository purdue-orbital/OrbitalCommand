use std::sync::Arc;
use num_complex::Complex;
use rustfft::Fft;

pub struct Demodulation{
    // Calculate the number of samples per a symbol
    pub(crate) samples_per_symbol: usize,

    // The rate the radio will sample at
    pub(crate) sample_rate: f32,


    pub(crate) mfsk_fft_size: usize,
    pub(crate) mfsk_fft_index_map: Vec<i32>,

    // pre-planned fft operation
    pub(crate) fft: Arc<dyn Fft<f32>>,

    // Pre allocated space for FFTs
    pub(crate) scratch: Vec<Complex<f32>>,
    

}
