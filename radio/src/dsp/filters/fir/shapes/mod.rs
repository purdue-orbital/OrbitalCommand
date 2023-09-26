use num_complex::Complex;

use crate::dsp::filters::fir::shapes::shape::Shape;

mod rectangle;
mod triangular;
mod shape;
mod welch;
mod hann;
mod nuttall_family;
mod flattop;
mod blackman;


/// This is different window shapes that can be used as a digital filter
pub enum WindowShapes {
    Rectangle,
    Triangle,
    Welch,
    Hann,
    BlackmanHarris,
    BlackmanNuttall,
    Nuttall,
    FlatTop,
    Blackman,
    //UltraSpherical,
    //PlanckBessel,
    //GapOpNuttal,
}

/// This will generate a window shape given by the enum
pub fn generate_shape(window_shape: WindowShapes, fft_size: usize, alpha: i16) -> Vec<Complex<f32>> {
    match window_shape {
        WindowShapes::Rectangle => rectangle::Rectangle::generate_shape(fft_size, alpha),
        WindowShapes::Triangle => triangular::Triangular::generate_shape(fft_size, alpha),
        WindowShapes::Welch => welch::Welch::generate_shape(fft_size, alpha),
        WindowShapes::Hann => hann::Hann::generate_shape(fft_size, alpha),
        WindowShapes::Nuttall => nuttall_family::NuttallBase::generate_shape(fft_size, alpha),
        WindowShapes::BlackmanNuttall => nuttall_family::NuttallBase::generate_shape(fft_size, alpha),
        WindowShapes::BlackmanHarris => nuttall_family::NuttallBase::generate_shape(fft_size, alpha),
        WindowShapes::FlatTop => flattop::FlatTop::generate_shape(fft_size, alpha),
        WindowShapes::Blackman => blackman::Blackman::generate_shape(fft_size, alpha),
        other => rectangle::Rectangle::generate_shape(fft_size, alpha),
    }
}

