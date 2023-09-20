//! Finite Impulse Response (FIR) is the reading and processing data of finite length.
//! Radio signals can be interpreted as finite and therefore FIR filters are very helpful. We could
//! also choose to read signals of a radio indefinitely and make changes and adjustments on the fly.
//! This is called Infinite Impulse Response (IIR). If this is the case, we would need a IIR filter
//! instead.

use num_complex::Complex;
use crate::dsp::filters::fir::shapes::WindowShapes;

pub mod shapes;

/// This filtering method uses "window functions" to remove data or frequencies we don't want.
pub struct Windowing{
    window: Vec<Complex<f32>>
}

impl Windowing {
    /// Generate a new windowing object
    pub fn new(window_shape: WindowShapes, fft_size:usize,alpha:i16) -> Windowing{
        // generate window
        let window = shapes::generate_shape(window_shape,fft_size,alpha);

        Windowing{window}
    }

    /// run/apply the filter onto a given set of data in place
    pub fn run(&self, arr:&mut [Complex<f32>]){
        for (index,x) in self.window.iter().enumerate(){
            arr[index] *= x;
        }
    }

}