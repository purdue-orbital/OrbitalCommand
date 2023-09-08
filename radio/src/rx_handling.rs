use std::sync::{Arc, RwLock};

use crate::IDENT;
use crate::tools::{bin_to_u8, u8_to_bin};

/*
Radio starts in "listen" mode where it starts looking for the signal identifier of IDENT

Once IDENT is found, we move to "read_length" mode where we then get the the next 16 bits which
tells us the length of the frame

We then record the the given length and write it into the buffer for reading by the main thread
 */

pub struct WindowHandler {
    pub window:Vec<u8>,
    pub window_flipped:Vec<u8>,
    pub recording:Vec<u8>,

    pub window_len: usize,

    pub currently_recording: bool,

    pub bit_counter: usize,

    pub ident: Vec<u8>,

    pub frame_len: u16,

    pub recording_len: usize,

    pub is_flipped: bool,
}

impl WindowHandler {
    pub fn new(ident_str_bin:&str) -> WindowHandler{

        let window_len = ident_str_bin.len() / 8;

        let ident = bin_to_u8(ident_str_bin);

        WindowHandler{
            window:vec![0;window_len],
            window_flipped:vec![0;window_len],

            recording:vec![0;65536],

            window_len,

            currently_recording: false,

            bit_counter: 0,

            recording_len: 0,

            frame_len: 0,

            ident,
            is_flipped:false,
        }
    }

    fn shift_and_carry(bin:&mut [u8],bit: u8){

        // set carry bit
        let mut carry = bit;

        // shift then add carry
        for x in bin.iter_mut().rev(){
            // save new carry bit
            let new_carry_bit = (*x >> 7) & 1;

            // shift and add carry bit
            *x = (*x << 1) ^ carry;

            // add new carry bit
            carry = new_carry_bit;
        }
    }

    pub fn add(&mut self, bin:&[u8]){
        
        if !self.currently_recording{
            WindowHandler::shift_and_carry(self.window.as_mut_slice(),bin[0]);
            WindowHandler::shift_and_carry(self.window_flipped.as_mut_slice(), !bin[0]);

            if self.window == self.ident{
                self.currently_recording = true;
            }

            // sometimes data comes in flipped, check for that case by having two data one flipped, the other not
            if self.window_flipped == self.ident{

                self.currently_recording = true;
                self.is_flipped = true;
            }

        }else {

            if self.bit_counter == 0{

                if self.recording_len == 2{
                    self.frame_len = ((self.recording[0] as u16) << 8) | (self.recording[1] as u16)
                }

                self.bit_counter = 8;
                self.recording_len += 1;
            }

            self.recording[self.recording_len - 1] <<= 1;

            if self.is_flipped{
                self.recording[self.recording_len - 1] += !bin[0];
            }else{
                self.recording[self.recording_len - 1] += bin[0];
            }

            self.bit_counter -= 1
        }
    }

    pub fn reset(&mut self){
        self.frame_len = 0;
        self.bit_counter = 0;
        self.currently_recording = false;
        self.recording_len = 0;
        self.is_flipped = false;
    }
}

pub struct RXLoop {
    len: u16,
    buffer: Arc<RwLock<Vec<Vec<u8>>>>,
}


impl RXLoop {
    pub fn new(buffer: Arc<RwLock<Vec<Vec<u8>>>>) -> RXLoop {
        RXLoop {
            len: 0,
            buffer,
        }
    }

    pub fn run(&mut self, window: &mut WindowHandler) {
        if window.frame_len != 0 && window.bit_counter == 0 && (window.recording_len- 2) >= window.frame_len as usize{

            if let Ok(mut x) = self.buffer.write(){
                x.push(window.recording.clone()[2..window.recording_len].to_owned());
            }

            window.reset()

        }
    }

}
