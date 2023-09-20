use num_complex::Complex;
use crate::dsp::filters::fir::shapes::shape::Shape;

pub struct Rectangle{

}

impl Shape for Rectangle{
    fn generate_shape(fft_size: usize, alpha:i16) -> Vec<Complex<f32>> {
        vec![Complex::new(1.0,1.0); fft_size]
    }
}