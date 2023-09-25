use num_complex::Complex;

pub trait Shape {
    /// This will generate a shape based on some function and return an array of complex f32.
    /// This array is then multiplied element-wise with the the data that underwent a FFT. In other
    /// words, this will produce values to create a filter in the frequency domain. Some shapes
    /// allow for a "alpha" variable which determines how drastic the filter will be applied. In
    /// those cases, the lower the alpha value, faster the processing but lower quality filtering
    /// and high the number, slower the filter will run but higher the quality wil be
    fn generate_shape(fft_size: usize, alpha: i16) -> Vec<Complex<f32>>;
}