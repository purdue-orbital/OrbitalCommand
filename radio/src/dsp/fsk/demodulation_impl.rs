use num_complex::Complex;
use rand::distributions::uniform::SampleBorrow;

use crate::dsp::fsk::modulation_impl::FSK_FREQUENCY2;
use crate::dsp::fsk::structs::demodulation::Demodulation;
use crate::dsp::tools::bi_signal_demodulation::bi_signal_demodulation;
use crate::dsp::tools::goertzel_algorithm::GoertzelAlgorithm;

impl Demodulation {
    pub fn new(samples_per_symbol: usize, sample_rate: f32) -> Demodulation {
        Demodulation {
            samples_per_symbol,
            sample_rate,
            goertzel_algorithm_fsk: GoertzelAlgorithm::new(
                samples_per_symbol as f32,
                sample_rate,
                FSK_FREQUENCY2,
            ),
        }
    }

    /// Demodulate a radio signal using FSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn run(&self, mut arr: Vec<Complex<f32>>) -> Vec<u8> {
        bi_signal_demodulation(
            arr.as_mut_slice(),
            &self.goertzel_algorithm_fsk,
            &(self.samples_per_symbol as f32 / 2.0),
            self.samples_per_symbol.borrow(),
        )
    }
}
