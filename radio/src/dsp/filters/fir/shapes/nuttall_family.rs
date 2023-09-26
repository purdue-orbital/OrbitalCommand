use num_complex::Complex;
use rand_distr::num_traits::Pow;
use crate::dsp::filters::fir::shapes::shape::Shape;
use std::f64::consts::PI;


fn nuttall_base_function(fft_size: usize, alpha: i16, coefficients: &[f32; 4]) -> Vec<Complex<f32>>
{
    let mut to_return: Vec<Complex<f32>> = Vec::with_capacity(fft_size);

    // Generate window
    for x in 0..fft_size{
        let value: f32 = coefficients[0] - (coefficients[1] * ((2 * PI * x) / fft_size as f32).cos()) + 
                                    (coefficients[2] * ((4 * PI * x) / fft_size as f32).cos()) -
                                    (coefficients[3] * (6 * PI * x) / fft_size).cos();
        to_return.push(Complex::new(value,value));
    }
    to_return
}
pub struct NuttallBase {
}

impl Shape for Nuttall{ 
    fn generate_shape(fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
        let coefficients: [f32; 4] = [0.355768, 0.487396, 0.144232, 0.012604];
        nuttall_base_function(fft_size, alpha, &coefficients)
    }
}

impl Shape for BlackmanNuttall{ 
    fn generate_shape(fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
        let coefficients: [f32; 4] = [0.3635819, 0.4891775, 0.1365995, 0.0106411];
        nuttall_base_function(fft_size, alpha, &coefficients)
    }
}

impl Shape for BlackmanHarris{ 
    fn generate_nuttall_shape(fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
        let coefficients: [f32; 4] = [0.35875, 0.48829, 0.14128, 0.01168];
        nuttall_base_function(fft_size, alpha, &coefficients)
    }
}