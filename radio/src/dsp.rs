use num_complex::Complex;


/// Calculate Amplitude
///
/// # Arguments
///
/// * `val` - Complex Radio Sample
pub fn amplitude(val : Complex<f32>) -> f32
{
    (val.re.powf(2.0) + val.im.powf(2.0)).sqrt()
}


/// Turns Complex Values From A Radio Wave Into A Array Of Amplitudes
/// This will return a Vec<f32> where each value is the amplitude
///
/// # Arguments
///
/// * `arr` - Array of Complex Radio Samples
pub fn amplitude_array(val : Vec<Complex<f32>>) -> Vec<f32>
{
    let mut out= Vec::new();

    for x in val
    {
        out.push(amplitude(x));
    }

    out
}


/// Generate Complex Radio Wave
///
/// # Arguments
///
/// * `frequency` - The Frequency Of The Wave
///
/// * `sample_rate` - The Sample Rate To Generate Wave
///
/// * `num_samples` - The Number Of Total Samples To To Make
pub fn generate_wave(frequency: f64, sample_rate: f64, num_samples: i32) -> Vec<Complex<f32>> {
    let mut arr: Vec<Complex<f32>> = Vec::new();

    // base
    let mut phi = 2.0 * std::f64::consts::PI * frequency;

    // time advance
    let mut adv = 1.0 / sample_rate;

    for x in 0..num_samples {
        arr.push(Complex::<f32>::new(
            (phi as f32 * adv as f32 * x as f32).cos(),
            (phi as f32 * adv as f32 * x as f32).sin(),
        ));
    }

    arr
}


// TODO: This is where the mod/demod functions will go