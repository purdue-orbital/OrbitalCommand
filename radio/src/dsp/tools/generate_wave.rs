use std::f32::consts::PI;

use num_complex::Complex;

/// Generate Complex Radio Wave
///
/// # Arguments
///
/// * `frequency` - The Frequency Of The Wave
///
/// * `sample_rate` - The Sample Rate To Generate Wave
///
/// * `num_samples` - The Number Of Total Samples To To Make
///
/// * `offset` - The Number Of Samples To Skip (IE: You already made 600 samples and want the next 100)
pub fn generate_wave(frequency: f32, sample_rate: f32, num_samples: i32, offset: i32, i_amplitude: f32, i_phase_offset: f32, q_phase_offset: f32) -> Vec<Complex<f32>> {
    let mut arr: Vec<Complex<f32>> = Vec::with_capacity(num_samples as usize);

    // base
    let phi = 2.0 * PI * frequency * (1.0 / sample_rate);

    for x in offset..offset + num_samples {
        arr.push(Complex::<f32>::new(
            i_amplitude * ((phi * x as f32) + i_phase_offset).cos(),
            i_amplitude * ((phi * x as f32) + q_phase_offset).sin(),
        ));
    }

    arr
}