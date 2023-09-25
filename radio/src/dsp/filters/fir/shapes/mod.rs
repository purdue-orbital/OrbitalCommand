use num_complex::Complex;

use crate::dsp::filters::fir::shapes::shape::Shape;

mod rectangle;
mod triangular;
mod shape;
mod welch;


/// This is different window shapes that can be used as a digital filter
pub enum WindowShapes {
    Rectangle,
    Triangle,
    Welch,
}

/// This will generate a window shape given by the enum
pub fn generate_shape(window_shape: WindowShapes, fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
    match window_shape {
        WindowShapes::Rectangle => rectangle::Rectangle::generate_shape(fft_size, alpha),
        WindowShapes::Triangle => triangular::Triangular::generate_shape(fft_size, alpha),
        WindowShapes::Welch => welch::Welch::generate_shape(fft_size, alpha),
    }
}

