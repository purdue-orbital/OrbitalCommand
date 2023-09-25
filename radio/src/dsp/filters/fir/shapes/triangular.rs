use num_complex::Complex;

use crate::dsp::filters::fir::shapes::shape::Shape;

pub struct Triangular {}

impl Shape for Triangular {
    fn generate_shape(fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
        let mut to_return = Vec::with_capacity(fft_size);

        let fft_div_2 = (fft_size / 2) as f32;

        // Generate window
        for x in 0..fft_size {
            let value = 1.0 - ((x as f32 - (fft_div_2)) / fft_div_2).abs();

            to_return.push(Complex::new(value, value));
        }

        to_return
    }
}