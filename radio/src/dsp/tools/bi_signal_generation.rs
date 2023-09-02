use num_complex::Complex;

/// Modulate a signal when when two signals are present
///
/// # Arguments
///
/// * `bin` - Binary String
/// * `zero_signal` - Pre generated signal on a '0' bit
/// * `one_signal` - Pre generated signal on a '1' bit
/// * `samples_per_symbol` - the number of samples per a symbol (in this case a number_of_symbols == bin.len()) (this can be calculated doing sample_rate / baud_rate)
pub fn bi_signal_modulation(bin: &[u8], zero_signal: &[Complex<f32>], one_signal: &[Complex<f32>], samples_per_symbol: usize) -> Vec<Complex<f32>> {

    // initialize vector
    let mut to_return = Vec::with_capacity(bin.len() * samples_per_symbol);

    // Generate wave
    for x in bin {
        for y in (0..8).rev() {
            if (x >> y) & 1 == 1 {
                to_return.extend_from_slice(one_signal)
            } else {
                to_return.extend_from_slice(zero_signal)
            }
        }
    }

    to_return
}