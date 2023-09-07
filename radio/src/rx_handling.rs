use std::sync::{Arc, RwLock};

use crate::IDENT;
use crate::tools::{bin_to_u8, flip_bin};

pub struct RXLoop {
    len: usize,
    buffer: Arc<RwLock<Vec<String>>>,
    counter: usize,
    arr: [fn(rxloop: &mut RXLoop, window: &mut String) -> u8; 4],
    flipped: String,
    was_flipped: bool,
}


impl RXLoop {
    pub fn new(buffer: Arc<RwLock<Vec<String>>>) -> RXLoop {
        RXLoop {
            len: 0,
            buffer,
            counter: 0,
            arr: [RXLoop::listen, RXLoop::sync, RXLoop::read_frame, RXLoop::record],
            flipped: String::new(),
            was_flipped: false,
        }
    }

    pub fn run(&mut self, window: &mut String) {
        self.flipped = flip_bin(window);


        self.counter = (self.counter + self.arr[self.counter](self, window) as usize) % 4;
    }

    fn listen(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.contains('1') {
            1
        } else {
            window.clear();

            0
        }
    }

    fn sync(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.contains(IDENT)
        {
            rxloop.was_flipped = false;

            window.clear();

            1
        } else if rxloop.flipped.contains(IDENT) {
            rxloop.was_flipped = true;

            window.clear();

            1
        } else if window.len() > 1000 {
            window.clear();

            3
        } else { 0 }
    }

    fn read_frame(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.len() >= 16 {
            if rxloop.was_flipped {
                rxloop.len = (((bin_to_u8(rxloop.flipped.as_str())[0] as u16) << 8) + bin_to_u8(rxloop.flipped.as_str())[1] as u16) as usize * 8usize;
            } else {
                rxloop.len = (((bin_to_u8(window.as_str())[0] as u16) << 8) + bin_to_u8(window.as_str())[1] as u16) as usize * 8usize;
            }


            window.clear();

            1
        } else { 0 }
    }

    fn record(rxloop: &mut RXLoop, window: &mut String) -> u8 {
        if window.len() >= rxloop.len {
            if rxloop.was_flipped {
                if let Ok(mut write_buf) = rxloop.buffer.write() {
                    write_buf.push(rxloop.flipped.clone());
                }
            } else if let Ok(mut write_buf) = rxloop.buffer.write() {
                write_buf.push(window.clone());
            }

            1
        } else {
            0
        }
    }
}
