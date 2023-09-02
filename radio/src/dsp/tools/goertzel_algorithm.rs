use std::f32::consts::PI;
use num_complex::Complex;
use rand_distr::num_traits::Pow;

/// Goertzel's Algorithm is a faster method of doing DFT compared to FFT as we're only calculating
/// the presence of one frequency. This is optimal if you demodulating a signal that is either
/// "there or not" such as FSK or OOK/ASK
#[derive(Clone)]
pub struct GoertzelAlgorithm{
    k:i32,
    w:f32,
    cosine:f32,
    sine:f32,
    coeff:f32,
}

impl GoertzelAlgorithm {
    pub fn new(len:f32, sample_rate:f32,target_frequency:f32) -> GoertzelAlgorithm {
        let k = (0.5 + ((len * target_frequency) / sample_rate)) as i32;
        let w = ((2.0 * PI) / len) * k as f32;
        let cosine = w.cos();

        GoertzelAlgorithm{
            k,
            w,
            cosine,
            sine: w.sin(),
            coeff: 2.0 * cosine,
        }
    }

    /// Default calculation
    pub fn run(&self, arr:&[Complex<f32>]) -> f32{

        // Initialize values
        let mut q_0:f32 = 0.0;
        let mut q_1:f32 = 0.0;
        let mut q_2:f32 = 0.0;

        // Run calculation
        for x in arr{
            q_0 = self.coeff * q_1 - q_2 + x.re;
            q_2 = q_1;
            q_1 = q_0;
        }

        // Calculate
        let real = q_1 - q_2 * self.cosine;
        let imag = q_2 * self.sine;

        // Return
        (real.pow(2) as f32 + imag.pow(2) as f32).sqrt()
    }

    /// Most cases the preferred option. This takes some liberty in final calculation
    pub fn run_optimized(&self, arr:&[Complex<f32>]) -> f32{
        // Initialize values
        let mut q_0:f32 = 0.0;
        let mut q_1:f32 = 0.0;
        let mut q_2:f32 = 0.0;

        // Run calculation
        for x in arr{
            q_0 = self.coeff * q_1 - q_2 + x.re;
            q_2 = q_1;
            q_1 = q_0;
        }

        (q_1.pow(2) as f32 + q_2.pow(2) as f32 - q_1 * q_2 * self.coeff).sqrt()

    }

}