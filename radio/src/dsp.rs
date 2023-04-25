use std::f32::consts::PI;

use num::pow::Pow;
use num_complex::Complex;
use rand_distr::Distribution;
use rand_distr::Normal;

/// This will add noise to a radio signal for testing
///
/// # Arguments
///
/// * `signal` - Complex Radio Samples to add simulated noise to
/// * `snr_db` - Signal to noise ratio. The lower the number, the more noise the signal is. (40 db is a good number to strive for)
pub fn gaussian_noise_generator(signal: &[Complex<f32>], snr_db: f32) -> Vec<Complex<f32>> {
    let snr = 10.0f32.powf(snr_db / 10.0); // calculate signal-to-noise ratio
    let signal_power = signal.iter().map(|x| x.norm_sqr()).sum::<f32>() / signal.len() as f32;
    let noise_power = signal_power / snr;
    let standard_deviation = (noise_power / 2.0).sqrt();

    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, standard_deviation).unwrap();

    signal.iter()
        .map(|&x| {
            let real = normal.sample(&mut rng);
            let imag = normal.sample(&mut rng);
            x + Complex::new(real, imag)
        })
        .collect()
}


/// Calculate Amplitude
///
/// # Arguments
///
/// * `val` - Complex Radio Sample
pub fn amplitude(val: Complex<f32>) -> f32
{
    (val.re.pow(2) as f32 + val.im.pow(2) as f32).sqrt()
}

/// Calculate Phase
///
/// # Arguments
///
/// * `val` - Complex Radio Sample
pub fn phase(val: Complex<f32>) -> f32
{
    let im = val.im;

    im.atan2(val.re)
}


/// Turns Complex Values From An Radio Wave Into A Array Of Amplitudes
/// This will return a Vec<f32> where each value is the amplitude
///
/// # Arguments
///
/// * `arr` - Array of Complex Radio Samples
pub fn amplitude_array(val: Vec<Complex<f32>>) -> Vec<f32>
{
    let mut out = Vec::new();

    for x in val
    {
        out.push(amplitude(x));
    }

    out
}


/// Turns Complex Values From A Radio Wave Into An Array Of Phases
/// This will return a Vec<f32> where each value is a phase
///
/// # Arguments
///
/// * `arr` - Array of Complex Radio Samples
pub fn phase_array(val: Vec<Complex<f32>>) -> Vec<f32>
{
    let mut out = Vec::new();

    for x in val
    {
        out.push(phase(x));
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
///
/// * `offset` - The Number Of Samples To Skip (IE: You already made 600 samples and want the next 100)
pub fn generate_wave(frequency: f32, sample_rate: f32, num_samples: i32, offset: i32, amplitude: f32) -> Vec<Complex<f32>> {
    let mut arr: Vec<Complex<f32>> = Vec::with_capacity(num_samples as usize);

    // base
    let phi = 2.0 * PI * frequency * (1.0 / sample_rate);

    for x in offset..offset + num_samples {
        arr.push(Complex::<f32>::new(
            amplitude * (phi * x as f32).cos(),
            amplitude * (phi * x as f32).sin(),
        ));
    }

    arr
}

/// Radio filters for digital signal processing
pub struct Filters {}

impl Filters {}

/// ASK Settings

/// Frequency of ask for "1"s
static ASK_FREQUENCY: f32 = 100.0;

/// Radio modulators for digital signal processing
pub struct Modulators {}

/// Radio demodulators for digital signal processing
pub struct Demodulators {}


/// Radio modulators for digital signal processing
impl Modulators {
    /// Modulate a radio signal using FSK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `baud_rate` - The number of symbols to send per a second (EX: baud_rate 100 = 100 bits a second)
    pub fn ask(bin: &str, sample_rate: f32, baud_rate: f32) -> Vec<Complex<f32>>
    {
        // Calculate the number of samples per a symbol
        let samples_per_symbol = (sample_rate / baud_rate) as usize;

        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * samples_per_symbol);

        let one_signal = generate_wave(ASK_FREQUENCY, sample_rate, samples_per_symbol as i32, 0, 1.0);
        let zero_signal = generate_wave(0.0, sample_rate, samples_per_symbol as i32, 0, 0.0);

        // Generate wave
        for x in bin.chars() {

            to_return.append((if x == '1' {one_signal.clone()} else {zero_signal.clone()}).as_mut());

        }

        to_return
    }

    // TODO: Although FSK is a great modulator, BPSK and subsequent QPSK are much more efficient
}

/// Radio demodulators for digital signal processing
impl Demodulators {
    /// Demodulate a radio signal using ASK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `baud_rate` - The number of symbols to send per a second (EX: baud_rate 100 = 100 bits a second)
    pub fn ask(mut arr: Vec<Complex<f32>>, sample_rate: f32, baud_rate: f32) -> String
    {
        // Calculate the number of samples per a symbol
        let samples_per_symbol = sample_rate / baud_rate;

        let symbol_threshold = samples_per_symbol / 2.0;

        // calculate the index to look at
        let fft_index = (ASK_FREQUENCY / (sample_rate / samples_per_symbol)).round() as usize;

        // this is scratch space for FFTs
        let mut scratch = vec![Complex::new(0.0, 0.0); arr.len()];
        let mut planner = rustfft::FftPlanner::new();

        let fft = planner.plan_fft_forward(samples_per_symbol as usize);
        fft.process_with_scratch(arr.as_mut_slice(), scratch.as_mut_slice());

        let mut out = String::with_capacity(arr.len() / samples_per_symbol as usize);

        for x in (fft_index..arr.len()).step_by(samples_per_symbol as usize) {

            out.push(if arr[x].re.abs() >= symbol_threshold {'1'} else {'0'});

        }

        out
    }

    // TODO: BPSK / QPSK Demodulator
}



