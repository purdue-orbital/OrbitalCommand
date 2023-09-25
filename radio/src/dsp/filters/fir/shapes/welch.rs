use num_complex::Complex;
use rand_distr::num_traits::Pow;

use crate::dsp::filters::fir::shapes::shape::Shape;

pub struct Welch {}

impl Shape for Welch {
    fn generate_shape(fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
        let mut to_return = Vec::with_capacity(fft_size);

        let fft_div_2 = (fft_size / 2) as f32;

        // Generate window
        for x in 0..fft_size {
            // this generates an error if no "as f32" is here
            let value = 1.0 - ((x as f32 - fft_div_2) / fft_div_2).pow(2) as f32;

            to_return.push(Complex::new(value, value));
        }


        to_return
    }
}