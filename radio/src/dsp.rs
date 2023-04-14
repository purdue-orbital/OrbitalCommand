use std::f64::consts::PI;
use num::pow::Pow;
use num::traits::real::Real;
use num_complex::Complex;
#[cfg(test)]
use plotters::prelude::*;
use rand::Rng;
use rand_distr::Normal;
use crate::tools::{moving_average, normalize, subtract_left_adjacent};
use rand_distr::Distribution;


/// This will add noise to a radio signal for testing
///
/// # Arguments
///
/// * `signal` - Complex Radio Samples to add simulated noise to
/// * `snr_db` - Signal to noise ratio. The lower the number, the more noise the signal is. (40 db is a good number to strive for)
pub(crate) fn gaussian_noise_generator(signal: &[Complex<f32>], snr_db: f32) -> Vec<Complex<f32>> {
    let snr = 10.0f32.powf(snr_db / 10.0); // calculate signal-to-noise ratio
    let signal_power = signal.iter().map(|x| x.norm_sqr()).sum::<f32>() / signal.len() as f32;
    let noise_power = signal_power / snr;
    let standard_deviation = (noise_power / 2.0).sqrt();

    let mut rng = rand::thread_rng();
    let mut normal = Normal::new(0.0, standard_deviation).unwrap();

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
    let mut im = val.im;

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
pub fn generate_wave(frequency: f64, sample_rate: f64, num_samples: i32, offset: i32) -> Vec<Complex<f32>> {
    let mut arr: Vec<Complex<f32>> = Vec::new();

    // base
    let mut phi = 2.0 * std::f64::consts::PI * frequency;

    // time advance
    let mut adv = 1.0 / sample_rate;

    for x in offset..offset + num_samples {
        arr.push(Complex::<f32>::new(
            (phi as f32 * adv as f32 * x as f32).cos(),
            (phi as f32 * adv as f32 * x as f32).sin(),
        ));
    }

    arr
}

/// Radio filters for digital signal processing
pub struct Filters{}

impl Filters {

}


/// FSK Settings

/// Wave frequency when binary is 0
static FSK_FREQUENCY1: f64 = 1e3;

/// Wave frequency when binary is 1
static FSK_FREQUENCY2: f64 = 10e3;


/// ASK Settings

/// Frequency of ask for "1"s
static ASK_FREQUENCY: f64 = 5e3;

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
    pub fn ask(bin: &str, sample_rate: f64, baud_rate : f64) -> Vec<Complex<f32>>
    {
        // Calculate the number of samples per a symbol
        let samples_per_symbol = sample_rate / baud_rate;

        // calculate the total number of samples
        let num_samples = bin.len() as f64 * samples_per_symbol;

        // initialize vector
        let mut toReturn = Vec::new();

        // Values stores if the current set of samples will represent aa 1 or a 0
        let mut bit = 1;

        // Convert binary string to an array of ints
        const RADIX: u32 = 10;
        let mut binary = bin.chars().map(|c| c.to_digit(RADIX).unwrap());

        // calculate first part of PHI for faster calculations
        let phi = 2.0 * PI * ASK_FREQUENCY;

        // Generate wave
        for x in 0..num_samples as i32{

            // switch the bits at the end of every set of symbols
            if x % samples_per_symbol as i32 == 0 {
                bit = binary.next().unwrap() as i32;
            }

            toReturn.push(Complex::new(bit as f32 * (phi * (x as f64 / sample_rate) as f64).cos() as f32, bit as f32 * (phi * (x as f64 / sample_rate) as f64).sin() as f32));

        }

        toReturn
    }

    /// # WIP
    /// Modulate a radio signal using FSK
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `sample_time` - The amount of time, in seconds, a wave samples for per a symbol (IE: 0.02 Seconds sample_time for 6 symbols = 0.12 seconds total)
    pub fn fsk(bin: &str, sample_rate: f64, sample_time : f64) -> Vec<Complex<f32>> {

        // Make the array that will get returned with the modulated signal
        let mut signal = Vec::new();

        // Get an array of all the chars in the string
        let mut chars = bin.chars().clone();

        let sample_size = sample_rate * sample_time;


        // Loop through each char in the string. We need the index number for creating
        for x in 0..bin.len() {

            // For each "1" bit, generate a wave with a higher frequency, else just use base frequency
            if chars.nth(0).unwrap() == '1' {
                signal.append(&mut generate_wave(FSK_FREQUENCY2, sample_rate, sample_size as i32, x as i32));
            } else {
                signal.append(&mut generate_wave(FSK_FREQUENCY2, sample_rate, sample_size as i32,x as i32));
            }
        }

        signal
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
    pub fn ask(arr : Vec<Complex<f32>>, sample_rate: f64, baud_rate : f64) -> String
    {
        let mut out = String::from("");

        // Calculate the number of samples per a symbol
        let samples_per_symbol = sample_rate / baud_rate;

        // convert the samples to an amplitude representation of the samples
        let mut amplitudes = amplitude_array(arr.clone());

        // preform moving average
        let avg = moving_average(amplitudes.clone(), samples_per_symbol as usize);

        // Normalize inputs
        let normal = normalize(avg.clone());

        // Turn inputs into binary array
        let test:Vec<i8> = normal.iter().enumerate().filter(|&(i, _)| i as i32  % samples_per_symbol as i32 == 0).map(|(_,&v)| v.round() as i8).collect();

        // Turn binary array into binary string
        for x in test.clone(){
            if x == 1{
                out.push('1');
            } else{
                out.push('0');
            }
        }

        out
    }


    ///# __WIP__
    ///
    /// Demodulate a radio signal using FSK
    /// # Arguments
    /// * `arr` - Array of radio samples to
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `sample_time` - The amount of time, in seconds, a wave samples for per a symbol (IE: 0.02 Seconds sample_time for 6 symbols = 0.12 seconds total)
    ///
    /// # Return
    /// This will return a two string concatenated First half it the received value, the second half are flipped bits
    ///
    pub fn fsk(arr : Vec<Complex<f32>>, sample_rate: f64, sample_time : f64) -> String {
        let mut toReturn = String::from("");

        // Calculate the phase difference for FSK for "0"s
        let phi1 = 2.0 * PI * FSK_FREQUENCY1 * (1.0 / sample_rate);
        let difference1 = amplitude(Complex::new(0 as f32,0 as f32)) - amplitude(Complex::new(phi1.cos() as f32,phi1.sin() as f32));

        // Calculate the phase difference for FSK for "1"s
        let phi2 = 2.0 * PI * FSK_FREQUENCY2 * (1.0 / sample_rate);
        let difference2 = amplitude(Complex::new(0 as f32,0 as f32)) - amplitude(Complex::new(phi2.cos() as f32,phi2.sin() as f32));

        let num_skip = sample_rate * sample_time;

        // Subtract the difference in phase changes
        let mut phases = subtract_left_adjacent(amplitude_array(arr));
        let mut it = phases.iter();

        while let Some(x) = it.next()
        {
            // If value is closer to being 0 than 1, set 0
            if (x - difference2).abs() > (x - difference1).abs(){
                toReturn.push('0');
            }else {
                toReturn.push('1');
            }

            it.nth((num_skip-1.0) as usize);
        }

        toReturn
    }

    // TODO: BPSK / QPSK Demodulator
}


/// This implementation will make graphs for visually analyzing radio waves
#[cfg(test)]
pub struct Graph {}

#[cfg(test)]
impl Graph {
    /// Graph a signal with respect of time
    pub fn time_graph(file_name: &str, arr: Vec<Complex<f32>>) -> Result<(), Box<dyn std::error::Error>> {

        // New Image
        let root = BitMapBackend::new(file_name, (640, 480)).into_drawing_area();

        // Set Background to White
        root.fill(&WHITE)?;

        // Set chart values
        let mut chart = ChartBuilder::on(&root)
            .caption("Time Graph", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0f32..arr.len() as f32, -1f32..1f32)?;

        // Draw graph values
        chart.configure_mesh().draw()?;

        // Create I (Real Part)  of the wave
        chart.draw_series(LineSeries::new(
            (0..arr.len()).map(|x| x as f32).map(|x| {
                (
                    x,
                    arr.get(x as usize).unwrap().re,
                )
            }),
            &RED,
        ))?
            // Set Legend
            .label("I")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));


        // Create Q (Imaginary Part) of the wave
        chart.draw_series(LineSeries::new(
            (0..arr.len()).map(|x| x as f32).map(|x| {
                (
                    x,
                    arr.get(x as usize).unwrap().im,
                )
            }),
            &BLUE,
        ))?
            // Set Legend
            .label("Q")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        // Draw labels to graph
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        // Send Graph to file
        root.present()?;

        Ok(())
    }

    // TODO: Make various graphs for analyzing radio waves
}


