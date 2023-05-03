use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Arc;

use num_complex::Complex;
use rand_distr::Distribution;
use rand_distr::Normal;
use rustfft::{Fft, FftPlanner};
use rustfft::num_traits::Pow;
use crate::tools::i32_to_bin;

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
/// (MFSK_BANDWIDTH / (BAUD_RATE * (2^MFSK_BITS_ENCODED))) == 1
///
/// and
///
/// MFSK_BANDWIDTH <= SAMPLE_RATE
///
/// This has to do with a limitation with rustfft and most likely the FFT algorithm as a whole
///

// the bandwidth in hz of MFSK
static MFSK_BANDWIDTH: f32 = 256e4;

// the number of bits represented by a symbol
static MFSK_BITS_ENCODED: i32 = 8;

// the number of samples that FFT will return per an evaluation (Higher this value is, more computation time but higher accuracy)
static MFSK_FFT_SIZE: usize = 1024;

///-------------------------------------------------------------------------------------------------
/// ASK Settings
///-------------------------------------------------------------------------------------------------

/// Frequency of ask for "1"s
static ASK_FREQUENCY: f32 = 100.0;

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

    symbol_threshold:usize,

    fsk_fft_index: usize,
    ask_fft_index: usize,

    mfsk_fft_index_map: Vec<i32>,

    // pre-planned fft operation
    fft: Arc<dyn Fft<f32>>,
}


/// Radio modulators for digital signal processing
impl Modulators {

    /// Create a modulation instance
    ///
    /// * `sample_rate` - The rate the __RADIO__ samples at in hz
    /// * `baud_rate` - The number of symbols to send per a second (EX: baud_rate 100 = 100 symbols a second)
    pub fn new(sample_rate: f32, baud_rate: f32) -> Modulators{
        let samples_per_symbol = (sample_rate / baud_rate) as usize;

        let transmission_window = MFSK_BANDWIDTH as i32 / 2.pow(MFSK_BITS_ENCODED as u32);

        // Generate frequency map
        let mut mfsk_freq_map = Vec::with_capacity(2.pow(MFSK_BITS_ENCODED as u32) as usize);
        let mut counter = 0.0;

        // Create frequency map
        for _ in 0..2.pow(MFSK_BITS_ENCODED as u32) as usize{
            mfsk_freq_map.push(generate_wave(counter, sample_rate, samples_per_symbol as i32, 0 , 1.0));

            counter += transmission_window as f32;
        }

        Modulators{
            samples_per_symbol,
            sample_rate,

            ask_on_signal: generate_wave(ASK_FREQUENCY, sample_rate, samples_per_symbol as i32, 0, 1.0),
            ask_off_signal: generate_wave(0.0, sample_rate, samples_per_symbol as i32, 0, 0.0),

            fsk_one_signal: generate_wave(FSK_FREQUENCY1, sample_rate, samples_per_symbol as i32, 0, 1.0),
            fsk_zero_signal: generate_wave(FSK_FREQUENCY2, sample_rate, samples_per_symbol as i32, 0, 1.0),

            mfsk_freq_map,
        }
    }

    /// Update sample rate and baud rate
    pub fn update(&mut self, sample_rate: f32, baud_rate: f32){

        self.samples_per_symbol = (sample_rate / baud_rate) as usize;
        self.sample_rate = sample_rate;

        self.ask_on_signal = generate_wave(ASK_FREQUENCY, sample_rate, self.samples_per_symbol as i32, 0, 1.0);
        self.ask_off_signal = generate_wave(0.0, sample_rate, self.samples_per_symbol as i32, 0, 0.0);

        self.fsk_one_signal = generate_wave(FSK_FREQUENCY1, sample_rate, self.samples_per_symbol as i32, 0, 1.0);
        self.fsk_zero_signal = generate_wave(FSK_FREQUENCY2, sample_rate, self.samples_per_symbol as i32, 0, 1.0);

        // Generate frequency map
        self.mfsk_freq_map = Vec::with_capacity(2.pow(MFSK_BITS_ENCODED as u32) as usize);
        let mut counter = 0.0;
        let transmission_window = MFSK_BANDWIDTH as i32 / 2.pow(MFSK_BITS_ENCODED as u32);

        // Create frequency map
        for _ in 0..2.pow(MFSK_BITS_ENCODED as u32) as usize{
            self.mfsk_freq_map.push(generate_wave(counter, self.sample_rate, self.samples_per_symbol as i32, 0 , 1.0));

            counter += transmission_window as f32;
        }
    }

    /// Modulate a radio signal using ASK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn ask(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * self.samples_per_symbol);

        // Generate wave
        for x in bin.chars() {

            to_return.append((if x == '1' {self.ask_on_signal.clone()} else {self.ask_off_signal.clone()}).as_mut());

        }

        to_return
    }

    /// Modulate a radio signal using FSK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn fsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * self.samples_per_symbol);

        // Generate wave
        for x in bin.chars() {

            to_return.append((if x == '1' {self.fsk_one_signal.clone()} else {self.fsk_zero_signal.clone()}).as_mut());

        }

        to_return
    }

    /// Modulate a radio signal using MFSK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn mfsk(&mut self, bin: &str) -> Vec<Complex<f32>>
    {
        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * self.samples_per_symbol);

        // Temporary var that will hold the bits to encode
        let mut hold = String::with_capacity(MFSK_BITS_ENCODED as usize);

        for x in bin.chars() {

            hold.push(x);

            if hold.len() == MFSK_BITS_ENCODED as usize{

                to_return.append(self.mfsk_freq_map[usize::from_str_radix(&hold, 2).unwrap()].clone().as_mut());

                hold.clear();
            }

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
    pub fn new(sample_rate: f32, baud_rate: f32) -> Demodulators{
        let samples_per_symbol = (sample_rate / baud_rate) as usize;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(samples_per_symbol);

        let symbol_threshold = samples_per_symbol / 2;

        // calculate the index to look at
        let fsk_fft_index = (FSK_FREQUENCY1 / (sample_rate / samples_per_symbol as f32)).round() as usize;
        let ask_fft_index = (ASK_FREQUENCY / (sample_rate / samples_per_symbol as f32)).round() as usize;


        // create index map for mfsk
        let mut mfsk_fft_index_map = vec![0; samples_per_symbol];
        let transmission_window = MFSK_BANDWIDTH as i32 / 2.pow(MFSK_BITS_ENCODED as u32);
        let fft_idk = sample_rate / samples_per_symbol as f32; // I don't know why we need to do this but this is how we can find what indexes go to what frequencies

        // create map
        if samples_per_symbol >= 2.pow(MFSK_BITS_ENCODED as u32) as usize {
            for x in 0..2.pow(MFSK_BITS_ENCODED as u32) as i32 {
                let index = ((x * transmission_window) as f32 / fft_idk).round() as usize;

                mfsk_fft_index_map[index] = x;
            }
        }


        Demodulators{ samples_per_symbol, sample_rate, symbol_threshold, fsk_fft_index, ask_fft_index, mfsk_fft_index_map, fft}
    }

    /// Update sample rate and baud rate
    pub fn update(&mut self, sample_rate: f32, baud_rate: f32){

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
    }

    /// Demodulate a radio signal using ASK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn ask(&mut self, mut arr: Vec<Complex<f32>>) -> String
    {
        // run fft
        let mut scratch = vec![Complex::new(0.0, 0.0); arr.len()];
        self.fft.process_with_scratch(arr.as_mut_slice(), scratch.as_mut_slice());

        let mut out = String::with_capacity(arr.len() / self.samples_per_symbol);

        for x in (self.ask_fft_index..arr.len()).step_by(self.samples_per_symbol) {

            out.push(if arr[x].re.abs() >= self.symbol_threshold as f32 {'1'} else {'0'});

        }

        out
    }

    /// Demodulate a radio signal using FSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn fsk(&mut self,mut arr: Vec<Complex<f32>>) -> String
    {
        // run fft
        let mut scratch = vec![Complex::new(0.0, 0.0); arr.len()];
        self.fft.process_with_scratch(arr.as_mut_slice(), scratch.as_mut_slice());

        let mut out = String::with_capacity(arr.len() / self.samples_per_symbol);

        for x in (self.fsk_fft_index..arr.len()).step_by(self.samples_per_symbol) {

            out.push(if arr[x].re.abs() >= self.symbol_threshold as f32 {'0'} else {'1'});

        }

        out
    }

    /// Demodulate a radio signal using MFSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn mfsk(&mut self, mut arr: Vec<Complex<f32>>) -> String
    {
        // run fft
        let mut scratch = vec![Complex::new(0.0, 0.0); arr.len()];
        self.fft.process_with_scratch(arr.as_mut_slice(), scratch.as_mut_slice());

        // Pre allocate space
        let mut out = String::with_capacity(arr.len() / self.samples_per_symbol);

        for x in (0..arr.len()).step_by(self.samples_per_symbol)  {
            for y in 0..self.samples_per_symbol {
                if arr[x + y].re >= self.symbol_threshold as f32{
                    out.push_str(i32_to_bin(self.mfsk_fft_index_map[y], MFSK_BITS_ENCODED as usize).as_str());

                    break;
                }
            }
        }

        out
    }

    // TODO: BPSK / QPSK Demodulator
}



