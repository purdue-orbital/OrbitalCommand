/*
This code is not intended to be compiled for production use

This code will test all radio components of the radio system to ensure the radio can work
 */
use colored::Colorize;
use crate::radio::Radio; // I like my colors okay


mod dsp;
mod tools;
mod radio;

/// This function will test if fsk works properly
///
/// Returns: Bool value of if fsk works properly
fn test_fsk() -> bool{
    // Modulation settings
    let sample_rate = 50e3;
    let sample_time = 0.02;

    // strings to modulate
                   //11010110101101011010110101101
    let s1 = "11001010101001010010011110101";
                   //110101101011010110101101011010110101101
    let s2 = "001010001010100010101111110000101010010";

    // Modulate values
    let mod_one = dsp::Modulators::fsk(s1, sample_rate, sample_time);
    let mod_two = dsp::Modulators::fsk(s2, sample_rate, sample_time);

    // Demodulate values
    let demod_one = dsp::Demodulators::fsk(mod_one, sample_rate, sample_time);
    let demod_two = dsp::Demodulators::fsk(mod_two, sample_rate, sample_time);

    println!("One: {}",demod_one);
    println!("Two: {}",demod_two);

    // return if demodulated values match
    String::from(s1) == demod_one && String::from(s2) == demod_two
}


/// This function will test if fsk works properly
///
/// Returns: Bool value of if fsk works properly
fn test_ask() -> bool{
    // Modulation settings
    let sample_rate = 100e3;
    let baud_rate = 10000.0;

    // strings to modulate
    let s1 = "11001010101001010010011110101";
    let s2 = "001010001010100010101111110000101010010";

    // Modulate values
    let mod_one = dsp::Modulators::ask(s1, sample_rate, baud_rate);
    let mod_two = dsp::Modulators::ask(s2, sample_rate, baud_rate);

    // add slight noise
    let noisy = dsp::gaussian_noise_generator(mod_two.as_slice(), 50.0);

    // Demodulate values
    let demod_one = dsp::Demodulators::ask(mod_one, sample_rate, baud_rate);
    let demod_two = dsp::Demodulators::ask(mod_two, sample_rate, baud_rate);
    let demod_three = dsp::Demodulators::ask(noisy, sample_rate, baud_rate);

    // return if demodulated values match
    String::from(s1) == demod_one && String::from(s2) == demod_two && String::from(s2) == demod_three
}

/// This function will benchmark ask and print it's noise score (Higher score is better) (100 is the highest)
///
/// For the more technically inclined, to get the SNR value, do 100 - score
fn bench_ask(){
    // Modulation settings
    let sample_rate = 100e3;
    let baud_rate = 10000.0;

    // initialize score
    let mut score = 100;

    // strings to modulate
    let s= "001010001010100010101111110000101010010";

    // Modulate value
    let arr = dsp::Modulators::ask(s, sample_rate, baud_rate);

    // benchmark
    for x in 0..100
    {
        // add noise
        let noisy = dsp::gaussian_noise_generator(arr.as_slice(), 100.0 - x as f32);

        // Demodulate
        let demod = dsp::Demodulators::ask(noisy, sample_rate, baud_rate);

        if demod.as_str() == s{
            score = x;
        } else {
            break;
        }
    }

    println!("[*] ASK Score: {}", score)
}


fn main() {
    //-----------------------------------------
    // Modulation tests
    //-----------------------------------------

    // Test FSK
    println!("[*] Testing ASK...");
    if !test_ask(){
        // Print error in red
        println!("{}","[!] ASK Failed!".red());

        return;
    }

    //-----------------------------------------
    // Modulation benchmarks
    //-----------------------------------------

    // Benchmark ask
    println!("[*] Benchmarking ASK...");
    bench_ask();

    //-----------------------------------------
    // Live tests
    //-----------------------------------------

    // Check if radio is connected
    let radio = Radio::new().unwrap();

    // If radio is connected, preform live tests
    if radio.is_connected(){
        println!("[!] Radio is connected. Running live test... ")

    }else {
        println!("[!] Radio is not connected. Skipping live test... ")
    }

    println!("[!] Done!")
}
