use num_complex::Complex;
use rustfft::FftPlanner;

use crate::dsp::mfsk::modulation_impl::{MFSK_BANDWIDTH, MFSK_BITS_ENCODED};
use crate::dsp::mfsk::structs::demodulation::Demodulation;

impl Demodulation {
    pub fn new(samples_per_symbol: usize, sample_rate: f32) -> Demodulation {
        let mfsk_fft_size = (MFSK_BANDWIDTH as usize - ((sample_rate / samples_per_symbol as f32) as i32 * (2 ^ MFSK_BITS_ENCODED)) as usize) + samples_per_symbol;


        // create index map for mfsk
        let mut mfsk_fft_index_map = vec![0; 2_i32.pow(MFSK_BITS_ENCODED as u32) as usize];
        let transmission_window = MFSK_BANDWIDTH as i32 / 2_i32.pow(MFSK_BITS_ENCODED as u32);
        let fft_idk = sample_rate / samples_per_symbol as f32; // I don't know why we need to do this but this is how we can find what indexes go to what frequencies

        if samples_per_symbol >= 2_i32.pow(MFSK_BITS_ENCODED as u32) as usize {
            for x in 0..2_i32.pow(MFSK_BITS_ENCODED as u32) {
                let index = ((x * transmission_window) as f32 / fft_idk).round() as usize;

                mfsk_fft_index_map[index] = x;
            }
        }

        let default_index =  0;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(samples_per_symbol);

        let scratch = vec![Complex::new(0.0, 0.0); 10000usize];


        Demodulation {
            samples_per_symbol,
            sample_rate,
            mfsk_fft_size,
            mfsk_fft_index_map,
            fft,
            scratch,
            default_index
        }
    }

    /// Demodulate a radio signal using MFSK
    ///
    /// # Arguments
    /// * `arr` - Array of radio samples to
    pub fn run(&self, arr: Vec<Complex<f32>>) -> Vec<u8>
    {
        let mut scratch = self.scratch.clone();
        let mut arr_clone = arr.clone();

        // run fft
        self.fft.process_with_scratch(arr_clone.as_mut_slice(), scratch.as_mut_slice());

        // Pre allocate space
        let mut out: Vec<u8> = Vec::with_capacity(arr.len() / self.samples_per_symbol);

        let chunks = arr_clone.chunks_exact(self.samples_per_symbol);

        let mut bin: u8 = 0;

        let mut counter = 0;

        let mut index = 0;

        let mut index_option = Some(self.default_index);

        for x in chunks {
            index_option = x.iter().position(|b| b.re >= (self.samples_per_symbol / 2) as f32);

            // default to an index upon error
            index = if let Some(x) = index_option{
                x
            }else {
                self.default_index
            };

            counter += 1;

            if MFSK_BITS_ENCODED < 8 {
                bin <<= MFSK_BITS_ENCODED as u8;
            } else {
                bin = 0;
            }

            bin += self.mfsk_fft_index_map[index] as u8;

            if counter * MFSK_BITS_ENCODED == 8 {
                out.push(bin);
                counter = 0;
            }
        }
        out
    }
}
