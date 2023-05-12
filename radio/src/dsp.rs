use std::{vec};
use std::f32::consts::PI;

use std::sync::Arc;

use num_complex::Complex;
use rand::distributions::uniform::SampleBorrow;
use rand_distr::Distribution;
use rand_distr::Normal;
use rayon::prelude::*;
use rustfft::{Fft, FftPlanner};
use rustfft::num_traits::Pow;

use crate::tools::{bin_char_arr_to_usize_unchecked, i32_to_char_bin};

/// Radio filters for digital signal processing
pub struct Filters {}

impl Filters {}

///-------------------------------------------------------------------------------------------------
/// Mod Settings
///-------------------------------------------------------------------------------------------------

static BUFFER_SIZE: f32 = 2048e4; // Pre allocated buffer size of values to return (For performance)

///-------------------------------------------------------------------------------------------------
/// Demod Settings
///-------------------------------------------------------------------------------------------------

static MAX_SYMBOLS: f32 = 2048e4; // Maximum numbers of samples that could be demodulated at once (For performance)

///-------------------------------------------------------------------------------------------------
/// FSK Settings
///-------------------------------------------------------------------------------------------------

static FSK_FREQUENCY1: f32 = 1e3;

static FSK_FREQUENCY2: f32 = 1e6;

///-------------------------------------------------------------------------------------------------
/// MFSK Settings
///-------------------------------------------------------------------------------------------------

///
/// You need to ensure that:
/// MFSK_BANDWIDTH == BAUD_RATE * (2^MFSK_BITS_ENCODED)
///
/// and
///
/// MFSK_BANDWIDTH <= SAMPLE_RATE
///
/// This has to do with a limitation with rustfft and most likely the FFT algorithm as a whole
///

// the bandwidth in hz of MFSK
static MFSK_BANDWIDTH: f32 = 2.56e6;

// the number of bits represented by a symbol
static MFSK_BITS_ENCODED: i32 = 8;

// the number of samples that FFT will return per an evaluation (Higher this value is, more computation time but higher accuracy)
static MFSK_FFT_SIZE: usize = 1024;

///-------------------------------------------------------------------------------------------------
/// ASK Settings
///-------------------------------------------------------------------------------------------------

/// Frequency of ask for "1"s
static ASK_FREQUENCY: f32 = 100.0;

/// Modulate a signal when when two signals are present
///
/// # Arguments
///
/// * `bin` - Binary String
/// * `zero_signal` - Pre generated signal on a '0' bit
/// * `one_signal` - Pre generated signal on a '1' bit
/// * `samples_per_symbol` - the number of samples per a symbol (in this case a number_of_symbols == bin.len()) (this can be calculated doing sample_rate / baud_rate)
#[inline]
pub fn bi_signal_modulation(bin: &str, zero_signal: &[Complex<f32>], one_signal: &[Complex<f32>], samples_per_symbol: usize) -> Vec<Complex<f32>> {

    // initialize vector
    let mut to_return = Vec::with_capacity(bin.len() * samples_per_symbol);

    // Generate wave
    for x in bin.chars() {
        to_return.extend(if x == '1' { one_signal } else { zero_signal });
    }

    to_return
}

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
pub fn bi_signal_demodulation(arr: &mut [Complex<f32>], index: &usize, threshold: &f32, scratch: &mut [Complex<f32>], fft: &Arc<dyn Fft<f32>>, samples_per_symbol: &usize) -> String {

    // run fft
    fft.process_with_scratch(arr, scratch);


    let mut out = String::with_capacity(arr.len() / samples_per_symbol);

    for x in (*index..arr.len()).step_by(*samples_per_symbol) {
        out.push(if arr[x].re.abs() >= *threshold { '1' } else { '0' });
    }

    out
}


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

/// Radio modulators for digital signal processing
#[derive(Clone)]
pub struct Modulators {
    // Calculate the number of samples per a symbol
    samples_per_symbol: usize,

    // The rate the radio will sample at
    sample_rate: f32,

    // ask pre-generated signals
    ask_on_signal: Vec<Complex<f32>>,
    ask_off_signal: Vec<Complex<f32>>,

    // Pre generated FSK signals
    fsk_one_signal: Vec<Complex<f32>>,
    fsk_zero_signal: Vec<Complex<f32>>,

    mfsk_freq_map: Vec<Vec<Complex<f32>>>,
}

/// Radio demodulators for digital signal processing
#[derive(Clone)]
pub struct Demodulators {
    // Calculate the number of samples per a symbol
    samples_per_symbol: usize,

    // The rate the radio will sample at
    sample_rate: f32,

    symbol_threshold: usize,

    fsk_fft_index: usize,
    ask_fft_index: usize,

    mfsk_fft_index_map: Vec<i32>,

    // pre-planned fft operation
    fft: Arc<dyn Fft<f32>>,

    // Pre allocated space for FFTs
    scratch: Vec<Complex<f32>>,
}


/// Radio modulators for digital signal processing
impl Modulators {
    /// Create a modulation instance
    ///
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `baud_rate` - The number of symbols to send per a second (EX: baud_rate 100 = 100 symbols a second)
    pub fn new(sample_rate: f32, baud_rate: f32) -> Modulators {

        // Create empty struct
        let mut out = Modulators { samples_per_symbol: 0, sample_rate: 0.0, ask_on_signal: vec![], ask_off_signal: vec![], fsk_one_signal: vec![], fsk_zero_signal: vec![], mfsk_freq_map: vec![] };

        // Update struct
        out.update(sample_rate, baud_rate);

        // return
        out
    }

    /// Update sample rate and baud rate
    pub fn update(&mut self, sample_rate: f32, baud_rate: f32) {
        self.samples_per_symbol = (sample_rate / baud_rate) as usize;
        self.sample_rate = sample_rate;

        self.ask_on_signal = generate_wave(ASK_FREQUENCY, sample_rate, self.samples_per_symbol as i32, 0, 1.0);
        self.ask_off_signal = generate_wave(0.0, sample_rate, self.samples_per_symbol as i32, 0, 0.0);

        self.fsk_one_signal = generate_wave(FSK_FREQUENCY2, sample_rate, self.samples_per_symbol as i32, 0, 1.0);
        self.fsk_zero_signal = generate_wave(FSK_FREQUENCY1, sample_rate, self.samples_per_symbol as i32, 0, 1.0);

        // Generate frequency map
        self.mfsk_freq_map = Vec::with_capacity(2.pow(MFSK_BITS_ENCODED as u32) as usize);
        let mut counter = 0.0;
        let transmission_window = MFSK_BANDWIDTH as i32 / 2.pow(MFSK_BITS_ENCODED as u32);

        // Create frequency map
        for _ in 0..2.pow(MFSK_BITS_ENCODED as u32) as usize {
            self.mfsk_freq_map.push(generate_wave(counter, self.sample_rate, self.samples_per_symbol as i32, 0, 1.0));

            counter += transmission_window as f32;
        }
    }

    /// Modulate a radio signal using ASK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn ask(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        bi_signal_modulation(bin, self.ask_off_signal.as_mut_slice(), self.ask_on_signal.as_mut_slice(), self.samples_per_symbol)
    }

    /// Modulate a radio signal using FSK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn fsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        bi_signal_modulation(bin, self.fsk_zero_signal.as_mut_slice(), self.fsk_one_signal.as_mut_slice(), self.samples_per_symbol)
    }

    /// Modulate a radio signal using MFSK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn mfsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * self.samples_per_symbol);

        for x in (0..bin.len()).step_by(MFSK_BITS_ENCODED as usize) {
            #[warn(clippy::needless_borrow)]  // This actually improves performance
                let signal = self.mfsk_freq_map[bin_char_arr_to_usize_unchecked((&bin[x..(x as i32 + MFSK_BITS_ENCODED) as usize]).chars())].as_ref();

            to_return.extend_from_slice(signal);
        }

        to_return
    }

    // TODO: Although FSK is a great modulator, BPSK and subsequent QPSK are much more efficient
}

/// Radio demodulators for digital signal processing
impl Demodulators {
    /// Create a demodulation instance
    ///
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `baud_rate` - The number of symbols to send per a second (EX: baud_rate 100 = 100 symbols a second)
    pub fn new(sample_rate: f32, baud_rate: f32) -> Demodulators {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(0);

        // Create empty struct
        let mut out = Demodulators { samples_per_symbol: 0, sample_rate: 0.0, symbol_threshold: 0, fsk_fft_index: 0, ask_fft_index: 0, mfsk_fft_index_map: vec![0], fft, scratch: vec![] };

        // Update struct
        out.update(sample_rate, baud_rate);

        // return
        out
    }

    /// Update sample rate and baud rate
    pub fn update(&mut self, sample_rate: f32, baud_rate: f32) {
        self.samples_per_symbol = (sample_rate / baud_rate) as usize;
        self.sample_rate = sample_rate;


        self.symbol_threshold = self.samples_per_symbol / 2;

        // calculate the index to look at
        self.fsk_fft_index = (FSK_FREQUENCY1 / (self.sample_rate / self.samples_per_symbol as f32)).round() as usize;
        self.ask_fft_index = (ASK_FREQUENCY / (self.sample_rate / self.samples_per_symbol as f32)).round() as usize;


        // create index map for mfsk
        self.mfsk_fft_index_map = vec![0; self.samples_per_symbol];
        let transmission_window = MFSK_BANDWIDTH as i32 / 2.pow(MFSK_BITS_ENCODED as u32);
        let fft_idk = self.sample_rate / self.samples_per_symbol as f32; // I don't know why we need to do this but this is how we can find what indexes go to what frequencies

        if self.samples_per_symbol >= 2.pow(MFSK_BITS_ENCODED as u32) as usize {
            for x in 0..2.pow(MFSK_BITS_ENCODED as u32) as i32 {
                let index = ((x * transmission_window) as f32 / fft_idk).round() as usize;

                self.mfsk_fft_index_map[index] = x;
            }
        }

        let mut planner = FftPlanner::new();
        self.fft = planner.plan_fft_forward(self.samples_per_symbol);

        self.scratch = vec![Complex::new(0.0, 0.0); MAX_SYMBOLS as usize];
    }

    /// Demodulate a radio signal using ASK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn ask(&mut self, mut arr: Vec<Complex<f32>>) -> String
    {
        bi_signal_demodulation(arr.as_mut_slice(), self.ask_fft_index.borrow(), (self.symbol_threshold as f32).borrow(), self.scratch.as_mut_slice(), &self.fft, self.samples_per_symbol.borrow())
    }

    /// Demodulate a radio signal using FSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn fsk(&mut self, mut arr: Vec<Complex<f32>>) -> String
    {
        bi_signal_demodulation(arr.as_mut_slice(), self.fsk_fft_index.borrow(), (self.symbol_threshold as f32).borrow(), self.scratch.as_mut_slice(), &self.fft, self.samples_per_symbol.borrow())
    }

    /// Demodulate a radio signal using MFSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn mfsk(&mut self, mut arr: Vec<Complex<f32>>) -> String
    {
        // // run fft
        self.fft.process_with_scratch(arr.as_mut_slice(), self.scratch.as_mut_slice());

        // Pre allocate space
        let mut out: Vec<char> = Vec::with_capacity(arr.len() / self.samples_per_symbol);

        let chunks = arr.chunks_exact(self.samples_per_symbol);
        for x in chunks {
            let index = x.iter().position(|b| b.re >= self.symbol_threshold as f32).unwrap();
            out.extend_from_slice(i32_to_char_bin(self.mfsk_fft_index_map[index], MFSK_BITS_ENCODED as usize).as_slice());
        }

        out.iter().collect()
    }

    // TODO: BPSK / QPSK Demodulator
}



