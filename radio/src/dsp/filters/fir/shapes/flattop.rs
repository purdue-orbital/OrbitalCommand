use num_complex::Complex;
use rand_distr::num_traits::Pow;
use crate::dsp::filters::fir::shapes::shape::Shape;
use std::f64::consts::PI;


pub struct FlatTop {

}

impl Shape for FlatTop{ 
    fn generate_shape(fft_size: usize, alpha:i16) -> Vec<Complex<f32>> {
        let mut to_return = Vec::with_capacity(fft_size);
        let coefficients: [f32; 5] = [0.21556895, 0.41663158, 0.277263158, 0.083578947, 0.006947368];
        // Generate window
        for x in 0..fft_size{
            let value: f32 = coefficients[0] - coefficients[1]((2 * PI * x) / fft_size).cos() + 
                coefficients[2]((4 * PI * x) / fft_size).cos() - 
                coefficients[3]((6 * PI * x) / fft_size).cos() + 
                coefficients[4]((8 * PI * x) / fft_size).cos();
            to_return.push(Complex::new(value,value));
        }

        to_return
    }
}