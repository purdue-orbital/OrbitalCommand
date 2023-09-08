use num_complex::Complex;
use rand::distributions::Distribution;
use rand_distr::Normal;

/// This will add noise to a radio signal for testing
///
/// # Arguments
///
/// * `signal` - Complex Radio Samples to add simulated noise to
/// * `snr_db` - Signal to noise ratio. The lower the number, the more noise the signal is. (40 db is a good number to strive for)
pub fn gaussian_noise_generator(signal: &[Complex<f32>], snr_db: f32) -> anyhow::Result<Vec<Complex<f32>>> {
    let snr = 10.0f32.powf(snr_db / 10.0); // calculate signal-to-noise ratio
    let signal_power = signal.iter().map(|x| x.norm_sqr()).sum::<f32>() / signal.len() as f32;
    let noise_power = signal_power / snr;
    let standard_deviation = (noise_power / 2.0).sqrt();

    let mut rng = rand::thread_rng();
    let normal = unsafe { Normal::new(0.0, standard_deviation).unwrap_unchecked() };

    Ok(
        signal.iter()
            .map(|&x| {
                let real = normal.sample(&mut rng);
                let imag = normal.sample(&mut rng);
                x + Complex::new(real, imag)
            })
            .collect()
    )
}