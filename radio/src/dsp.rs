use num_complex::Complex;
#[cfg(test)]
use plotters::prelude::*;
use crate::tools::subtract_left_adjacent;

/// Calculate Amplitude
///
/// # Arguments
///
/// * `val` - Complex Radio Sample
pub fn amplitude(val: Complex<f32>) -> f32
{
    (val.re.powf(2.0) + val.im.powf(2.0)).sqrt()
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
/// This will return a Vec<f32> where each value is the amplitude
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

/// Radio modulators for digital signal processing
pub struct Modulators {}

/// Radio demodulators for digital signal processing
pub struct Demodulators {}

/// Radio modulators for digital signal processing
impl Modulators {
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
                signal.append(&mut generate_wave(3e3, sample_rate, sample_size as i32, x as i32));
            } else {
                signal.append(&mut generate_wave(1e3, sample_rate, sample_size as i32,x as i32));
            }
        }

        signal
    }

    // TODO: Although FSK is a great modulator, BPSK and subsequent QPSK are much more efficient
}

/// Radio demodulators for digital signal processing
impl Demodulators {
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

        // counter for loop later
        let mut counter = 0.0;

        // The number of values to skip by
        let mut skip = sample_rate * sample_time;

        // A hold value for demodding later
        let mut previous = 0.0;

        // This value flips with the input values
        let mut one = false;

        // String to return once values are demodulated
        let mut out = String::new();

        // Get the phases in the array
        let mut phases = phase_array(arr);

        // Subtract adjacent values
        let mut modified = subtract_left_adjacent(phases);

        // Demod
        while (counter as usize) < modified.len(){

            // if the shift in frequency is large, mark as one, else zero
            if (modified.clone().get(counter as usize).unwrap() - previous).abs() > 0.05 {
                one = !one;
            }
            if one {
                out.push('1');
            }else{
                out.push('0');
            }

            // Save this value
            previous = *modified.clone().get(counter as usize).unwrap();

            // increase counter
            counter += skip;
        }

        out
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


