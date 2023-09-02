use num_complex::Complex;
use crate::dsp::qpsk::structs::demodulation::Demodulation;

impl Demodulation {

    pub fn new(samples_per_symbol: usize, sample_rate:f32)->Demodulation{
        Demodulation{samples_per_symbol, sample_rate}
    }


    /// Demodulate a radio signal using QPSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn run(&self, arr: Vec<Complex<f32>>) -> Vec<u8>
    {
        let mut to_return = Vec::with_capacity(arr.len() / self.samples_per_symbol);

        let mut bin: u8 = 0;
        let mut counter = 0;
        let step = self.samples_per_symbol / 2;

        for x in (step..arr.len()).step_by(self.samples_per_symbol) {
            counter += 2;
            bin <<= 2;

            // evaluate
            bin ^=
                if arr[x].re.is_sign_positive() {
                    if arr[x].im.is_sign_positive() {
                        3
                    } else {
                        2
                    }
                } else if arr[x].im.is_sign_positive() {
                    1
                } else {
                    0
                }
            ;

            if counter == 8 {
                to_return.push(bin);
                counter = 0;
            }
        }

        to_return
    }
}
