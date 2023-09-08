use std::sync::{Arc, RwLock};
use radio::{bits_per_symbol, demodulation, IDENT, modulation};
use radio::dsp::{Demodulators, Modulators};
use radio::rx_handling::{RXLoop, WindowHandler};
use radio::tools::{bin_to_u8, u8_to_bin};

static SAMPLE_RATE: f32 = 1e5;
static BAUD_RATE: f32 = 1e4;

fn add_data_bit_by_bit(window:&mut WindowHandler, bin: Vec<u8>){
    for x in 0..((bin.len() * 8) / bits_per_symbol() as usize){
        let shifted = bin[x / 8] >> (7 - (x % 8)) & 1;

        window.add(&[shifted])
    }
}

fn flip_u8s(bin:&[u8]) -> Vec<u8>{

    let mut to_return = Vec::new();

    for &x in bin{
        to_return.push(!x)
    }

    to_return
}

#[test]
pub fn window(){
    let samples_per_symbol = (SAMPLE_RATE / BAUD_RATE) as usize;

    let mods = Modulators::new(samples_per_symbol, SAMPLE_RATE);
    let demods = Demodulators::new(samples_per_symbol, SAMPLE_RATE);

    let fake_buffer = Arc::new(RwLock::new(Vec::new()));
    let mut rxloop = RXLoop::new(fake_buffer.clone());

    // Mod, Demod IDENT
    let mut window = WindowHandler::new(IDENT);
    let ident_arr = modulation(&mods,bin_to_u8(IDENT).as_slice());
    let ident_arr_demoded = demodulation(&demods,ident_arr.clone());

    // Add demoded ident
    add_data_bit_by_bit(&mut window, ident_arr_demoded);

    rxloop.run(&mut window);

    // ensure IDENT was detected
    assert!(window.currently_recording);

    // add 8 (number of bytes to add later)
    add_data_bit_by_bit(&mut window, vec![0, 8]);

    // ensure length is detected
    assert_eq!(window.frame_len,8);

    // add 8 bytes of data
    let test_data = vec![24,241,58,1,0,3,91,2];
    add_data_bit_by_bit(&mut window, test_data.clone());

    // run rx loop once
    rxloop.run(&mut window);

    // see if fake buffer has the test data
    assert_eq!(fake_buffer.read().unwrap()[0],test_data);

}


#[test]
pub fn window_flipped(){
    let samples_per_symbol = (SAMPLE_RATE / BAUD_RATE) as usize;

    let mods = Modulators::new(samples_per_symbol, SAMPLE_RATE);
    let demods = Demodulators::new(samples_per_symbol, SAMPLE_RATE);

    // Mod, Demod IDENT
    let mut window = WindowHandler::new(IDENT);

    let mut hold = bin_to_u8(IDENT);
    flip_u8s(hold.as_mut_slice());

    let ident_arr = modulation(&mods,hold.as_slice());
    let ident_arr_demoded = demodulation(&demods,ident_arr.clone());
    let ident_arr_demoded_flipped = flip_u8s(ident_arr_demoded.as_slice());

    // Add demoded ident
    add_data_bit_by_bit(&mut window, ident_arr_demoded_flipped);

    // ensure IDENT was detected
    assert!(window.currently_recording);

    // add 8 (number of bytes to add later)
    add_data_bit_by_bit(&mut window, vec![!0, !8]);

    // ensure length is detected
    assert_eq!(window.frame_len,8);

    // add 8 bytes of data
    let test_data = vec![24,241,58,1,0,3,91,2];
    let test_data_flipped = flip_u8s(test_data.as_slice());
    add_data_bit_by_bit(&mut window, test_data_flipped.clone());

    // run rx loop once
    let fake_buffer = Arc::new(RwLock::new(Vec::new()));
    let mut rxloop = RXLoop::new(fake_buffer.clone());
    rxloop.run(&mut window);

    // see if fake buffer has the test data
    assert_eq!(fake_buffer.read().unwrap()[0],test_data);

}