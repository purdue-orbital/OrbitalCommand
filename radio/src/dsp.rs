use num_complex::Complex;

pub fn generate_wave(frequency: f64, sample_rate: f64, num_samples: i32) -> Vec<Complex<f32>>
{
    let mut arr: Vec<Complex<f32>> = Vec::new();

    // base
    let mut phi = 2.0 * std::f64::consts::PI * frequency;

    // time advance
    let mut adv = 1.0 / sample_rate;

    for x in 0..num_samples {
        arr.push(Complex::<f32>::new((phi as f32 * adv as f32 * x as f32).cos(), (phi as f32 * adv as f32 * x as f32).sin()));
    }

    arr
}