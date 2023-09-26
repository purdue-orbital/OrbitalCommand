use num_complex::Complex;
use rand_distr::num_traits::Pow;
use crate::dsp::filters::fir::shapes::shape::Shape;
use std::f64::consts::PI;


pub struct FlatTop {

}

impl Shape for FlatTop{ 
    fn generate_shape(fft_size: usize, alpha:i16) -> Vec<Complex<f32>> {
        let mut to_return = Vec::with_capacity(fft_size);
        let coefficients: [f32; 3] = [alpha, 1/2, alpha/2];
        // Generate window
        for x in 0..fft_size{
            let value: f32 = coefficients[0] - coefficients[1]((2 * PI * x)/fft_size).cos() + coefficients[2]((4 * PI * x)/fft_size).cos();
            to_return.push(Complex::new(value,value));
        }

        to_return
    }
}