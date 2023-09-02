use num_complex::Complex;
use crate::dsp::mfsk::structs::modulation::Modulation;
use crate::dsp::tools::generate_wave::generate_wave;


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
pub static MFSK_BANDWIDTH: f32 = 2.56e6;

// the number of bits represented by a symbol
pub static MFSK_BITS_ENCODED: i32 = 8;

// the number of samples that FFT will return per an evaluation (Higher this value is, more computation time but higher accuracy)
pub static MFSK_FFT_SIZE: usize = 1024;


impl Modulation {

    pub fn new(samples_per_symbol: usize, sample_rate:f32)->Modulation{

        // Generate frequency map
        let mut mfsk_freq_map = Vec::with_capacity(2_i32.pow(MFSK_BITS_ENCODED as u32) as usize);
        let mut counter = 0.0;
        let transmission_window = MFSK_BANDWIDTH as i32 / 2_i32.pow(MFSK_BITS_ENCODED as u32);

        // Create frequency map for MFSK
        for _ in 0..2_i32.pow(MFSK_BITS_ENCODED as u32) as usize {
            mfsk_freq_map.push(generate_wave(counter, sample_rate, samples_per_symbol as i32, 0, 1.0,0.0,0.0));

            counter += transmission_window as f32;
        }
        
        Modulation{samples_per_symbol,sample_rate, mfsk_freq_map }
    }


    /// Modulate a radio signal using MFSK
    ///
    /// # Arguments
    /// * `bin` - String of binary bits (ONLY 1s & 0s) to modulate (AKA Symbols)
    pub fn run(&self, bin: &[u8]) -> Vec<Complex<f32>>
    {
        // initialize vector
        let mut to_return = Vec::with_capacity(bin.len() * self.samples_per_symbol);

        for &x in bin {
            #[warn(clippy::needless_borrow)]  // This actually improves performance
                let signal = self.mfsk_freq_map[x as usize].as_ref();

            to_return.extend_from_slice(signal);
        }

        to_return
    }
}