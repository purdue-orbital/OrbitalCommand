use num_complex::Complex;

use crate::dsp::tools::goertzel_algorithm::GoertzelAlgorithm;

/// Demodulate a signal when when two signals are present
///
/// # Arguments
///
/// * `arr` - Array of complex values
/// * `index` - Index in fft to find if '1' is present
/// * `threshold` - The number of samples of a 1 signal to evaluate as '1' bit (defaults to zero if below this threshold)
/// * `scratch` - Scratch space for fft calculation (for performance)
/// * `samples_per_symbol` - the number of samples per a symbol (in this case a number_of_symbols == bin.len()) (this can be calculated doing sample_rate / baud_rate)
#[inline]
pub fn bi_signal_demodulation(arr: &mut [Complex<f32>], algo: &GoertzelAlgorithm, threshold: &f32, samples_per_symbol: &usize) -> Vec<u8> {
    let mut out = Vec::new();

    let mut counter = 0;

    let mut bin: u8 = 0;

    for x in (0..arr.len()).step_by(*samples_per_symbol) {
        counter += 1;
        bin <<= 1;

        if algo.run_optimized(&arr[x..x + *samples_per_symbol]) >= *threshold {
            bin += 1;
        }

        if counter == 8 {
            out.push(bin);
            counter = 0;
        }
    }

    if counter > 0{
        out.push(bin);
    }

    out
}