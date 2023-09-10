use std::sync::{Arc, RwLock};
use crate::tools::{bin_to_u8};

/*
Radio starts in "listen" mode where it starts looking for the signal identifier of IDENT

Once IDENT is found, we move to "read_length" mode where we then get the the next 16 bits which
tells us the length of the frame

We then record the the given length and write it into the buffer for reading by the main thread
 */

/// This is a helper object that handles new bit(s) being passed to it and dynamically deciding to
/// record the incoming data or to throw it out
pub struct WindowHandler {
    /// This is the "view" that the radio has to find the IDENT. Once found, we then start recording
    pub window:Vec<u8>,

    /// This like window but all bits are flipped. This lets us check if IDENT is flipped
    pub window_flipped:Vec<u8>,

    /// This is the array that holds the recording of the transmission. At the end of transmission,
    /// we then write this buffer to the read buffer to be read by the main thread.
    pub recording:Vec<u8>,

    /// This is a bool that returns true if the IDENT was found and we're sending bits into
    /// recording
    pub currently_recording: bool,

    /// This is a counter that tells us what bit we're on in the last u8 value in the recording.
    /// When this reaches 0, we se the value back to 8 and add a new u8 value into the recording
    pub bit_counter: usize,

    /// This is a bin array version of IDENT which is a binary string. IDENT is a string for ease of
    /// visualization and this array then makes that visualization into something more practical.
    pub ident: Vec<u8>,

    /// This is the length of the frame extracted from the frame header. This is what tells us how
    /// long the radio should listen in the recording
    pub frame_len: u16,

    /// This is copy of the current index. Since the recording array is a fixed length of u16::MAX,
    /// we  use this to tell it where to add the data in the array
    pub recording_len: usize,

    /// This bool value returns true if the IDENT we recorded was in the flipped array, This means
    /// the transmission appears to have been flipped and we'll correct this by flipping all
    /// incoming bits in the recording.
    pub is_flipped: bool,
}

impl WindowHandler {
    /// This creates a new Window handler object that helps and streamlines the RX Loop and
    /// processing inbound data
    pub fn new(ident_str_bin:&str) -> WindowHandler{

        let window_len = ident_str_bin.len() / 8;

        let ident = bin_to_u8(ident_str_bin);

        let mut out = WindowHandler{
            window:vec![0;window_len],
            window_flipped:vec![0;window_len],

            recording:vec![0;65537],

            currently_recording: false,

            bit_counter: 0,

            recording_len: 0,

            frame_len: 0,

            ident,
            is_flipped:false,
        };

        out.reset();

        out
    }

    fn shift_and_carry(bin:&mut [u8],bit: u8){

        // set carry bit
        let mut carry = bit & 1;

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

    /// This will add and process byte that is passed to it decide to record it or pass it to the
    /// window
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
            self.recording[self.recording_len - 1] <<= 1;

            if self.is_flipped{
                self.recording[self.recording_len - 1] ^= !bin[0] & 1;
            }else{
                self.recording[self.recording_len - 1] ^= bin[0] & 1;
            }


            self.bit_counter -= 1;


            if self.bit_counter == 0{

                if self.recording_len == 2{
                    self.frame_len = ((self.recording[0] as u16) << 8) | (self.recording[1] as u16)
                }

                self.bit_counter = 8;
                self.recording_len += 1;
            }

        }
    }

    /// This will reset the the values to starting values and is used after we have recorded a
    /// inbound signal
    pub fn reset(&mut self){
        self.frame_len = 0;
        self.bit_counter = 8;
        self.currently_recording = false;
        self.recording_len = 1;
        self.is_flipped = false;
    }
}

/// This object handles the logic of inbound signals in the RX loop. THis makes the code look
/// cleaner and less cluttered
pub struct RXLoop {
    buffer: Arc<RwLock<Vec<Vec<u8>>>>,
}


impl RXLoop {
    /// This will create new RX loop by passing it the buffer that the main thread will listen on
    /// for new signals
    pub fn new(buffer: Arc<RwLock<Vec<Vec<u8>>>>) -> RXLoop {
        RXLoop {
            buffer,
        }
    }

    /// This runs the logic for on loop and is called once per a loop
    pub fn run(&mut self, window: &mut WindowHandler) {
        if window.frame_len != 0 && window.bit_counter == 8 && (window.recording_len - 2) >= window.frame_len as usize{

            unsafe {
                self.buffer.write().unwrap_unchecked()
                    .push(
                        window.recording.clone()
                            [2..window.recording_len - 1]
                            .to_owned()
                    );
            }

            window.reset()
        }
    }

}
