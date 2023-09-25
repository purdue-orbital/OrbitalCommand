//! Finite Impulse Response (FIR) is the reading and processing data of finite length.
//! Radio signals can be interpreted as finite and therefore FIR filters are very helpful. We could
//! also choose to read signals of a radio indefinitely and make changes and adjustments on the fly.
//! This is called Infinite Impulse Response (IIR). If this is the case, we would need a IIR filter
//! instead.

use std::sync::Arc;

use num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::dsp::filters::fir::shapes::WindowShapes;

pub mod shapes;

/// This filtering method uses "window functions" to remove data or frequencies we don't want.
pub struct Windowing {
    window: Vec<Complex<f32>>,
    forward_fft: Arc<dyn Fft<f32>>,
    inverse_fft: Arc<dyn Fft<f32>>,
    scratch_space: Vec<Complex<f32>>,
}

impl Windowing {
    /// Generate a new windowing object
    pub fn new(window_shape: WindowShapes, fft_size: usize, alpha: i16) -> Windowing {
        // generate window
        let window = shapes::generate_shape(window_shape, fft_size, alpha);

        // create wave settings
        let mut fft: FftPlanner<f32> = FftPlanner::new();
        let forward = fft.plan_fft_forward(fft_size);
        let reverse = fft.plan_fft_inverse(fft_size);

        Windowing { window, forward_fft: forward, inverse_fft: reverse, scratch_space: vec![Complex::new(0.0, 0.0); fft_size] }
    }

    /// run/apply the filter onto a given set of data in place
    pub fn run(&mut self, arr: &mut [Complex<f32>]) {

        // preform fft
        self.forward_fft.process_with_scratch(arr, self.scratch_space.as_mut_slice());

        // apply filter and normalization
        for (index, x) in self.window.iter().enumerate() {
            arr[index] = Complex::new(arr[index].re * x.re, arr[index].im * x.im) / self.scratch_space.len() as f32;
        }

        // preform inverse operation
        self.inverse_fft.process_with_scratch(arr, self.scratch_space.as_mut_slice());
    }
}