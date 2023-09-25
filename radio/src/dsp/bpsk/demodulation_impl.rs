use num_complex::Complex;

use crate::dsp::bpsk::structs::demodulation::Demodulation;

impl Demodulation {
    pub fn new(samples_per_symbol: usize, sample_rate: f32) -> Demodulation {
        Demodulation { samples_per_symbol, sample_rate }
    }

    /// Demodulate a radio signal using BPSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn run(&self, arr: Vec<Complex<f32>>) -> Vec<u8>
    {
        let mut to_return = Vec::with_capacity(self.samples_per_symbol);

        let mut bin: u8 = 0;
        let mut counter = 0;


        for x in (0..arr.len()).step_by(self.samples_per_symbol) {
            bin <<= 1;
            counter += 1;

            let mut to_sum = 0.0;

            for y in 0..self.samples_per_symbol - 1 {
                to_sum += arr[x + y].re;
            }

            if to_sum.is_sign_negative() {
                bin ^= 1;
            }

            if counter == 8 {
                to_return.push(bin);
                counter = 0;
                bin = 0;
            }
        }

        if counter > 0 {
            to_return.push(bin);
        }

        to_return
    }
}
