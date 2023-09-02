use crate::dsp::tools::goertzel_algorithm::GoertzelAlgorithm;

pub struct Demodulation {
    // Calculate the number of samples per a symbol
    pub(crate) samples_per_symbol: usize,

    // The rate the radio will sample at
    pub(crate) sample_rate: f32,

    pub(crate) goertzel_algorithm_fsk: GoertzelAlgorithm,
}
