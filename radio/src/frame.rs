//! This file contains the Frame object struct and implementation. Frame is the lowest OSI level
//! building block which is best handed and managed by the system itself. This particular frame
//! header implements CCSDS SDLP standard which is the standard frame header for space communication


use lazy_static::lazy_static;

use crate::{AMBLE, IDENT};
use crate::dsp::viterbi::decode::DecoderState;
use crate::dsp::viterbi::encode::EncoderState;
use crate::tools::bin_to_u8;

lazy_static! {
    pub static ref IDENT_VEC: Vec<u8> = bin_to_u8(IDENT);
    pub static ref AMBLE_VEC: Vec<u8> = bin_to_u8(AMBLE);
}

/// The Frame design implemented here is CCSDS SDLP which is specifically designed for use in
/// spacecraft and space bound communication
///
/// Here is the official standard: <https://public.ccsds.org/Pubs/132x0b3.pdf>
pub struct Frame {
    //--------------------------------
    // Transfer Frame Primary Header
    //--------------------------------

    // 2 bits
    version_number: u8,

    // 10 bits
    spacecraft_id: u16,

    // 3 bits
    virtual_channel_id: u8,

    // 1 bits
    ocf: bool,

    // 8 bits
    master_frame_count: u8,

    // 8 bits
    virtual_frame_count: u8,

    // 16 bits
    data_status: u16,

    /// this is the data that follows the frame header (the actual data being sent/received)
    pub data: Vec<u8>,

    amble: Vec<u8>,
    ident: Vec<u8>,

    pub has_ident: bool,
    pub is_complete: bool,

    len: u16,
}

impl Frame {
    /// Create a new frame object given data the will be encapsulated by the frame
    pub fn new(bytes: &[u8]) -> Frame {
        Frame {
            version_number: 0,
            spacecraft_id: 0,
            virtual_channel_id: 0,
            ocf: false,
            master_frame_count: 0,
            virtual_frame_count: 0,
            data_status: 0,
            data: bytes.to_vec(),
            amble: AMBLE_VEC.clone(),
            ident: IDENT_VEC.clone(),
            has_ident: true,
            is_complete: true,
            len: bytes.len() as u16,
        }
    }

    fn encode(bin: &[u8]) -> Vec<u8> {
        // let mut encoder: EncoderState<u8> = EncoderState::default();
        // encoder.push_slice(bin)

        bin.to_vec()
    }

    fn decode(bin: &[u8], expected_len: usize) -> Vec<u8> {
        // let mut decode = DecoderState::new(expected_len);
        // decode.push_slice(bin);
        // decode.read()

        bin.to_vec()
    }

    /// Turn a string into frame segments (if any)
    pub fn from(data: &[u8]) -> Frame
    {
        let mut new_frame = Frame::new(&[]);

        new_frame.has_ident = false;
        new_frame.is_complete = false;

        let ident_length_bytes = new_frame.ident.len();

        // safety check
        //if data.len() % 2 == 1 { return new_frame; };

        let decoded = Frame::decode(data, data.len() / 2);

        if decoded.len() >= ident_length_bytes && &decoded[..ident_length_bytes] == IDENT_VEC.as_slice() {
            new_frame.has_ident = true;

            if decoded.len() >= ident_length_bytes + 2 {
                new_frame.len = (decoded[ident_length_bytes] as u16) << 8 | decoded[ident_length_bytes + 1] as u16;

                if decoded.len() == ident_length_bytes + 2 + new_frame.len as usize {
                    new_frame.is_complete = true;

                    new_frame.data = decoded[ident_length_bytes + 2..].to_vec();
                }
            }
        }


        new_frame
    }

    /// This will assemble the frame header and make it ready to be transmitted
    pub fn assemble(&self) -> Vec<u8> {
        let len_bin = &[(self.len >> 8) as u8, self.len as u8];
        let mut hold = Vec::new();

        hold.extend_from_slice(self.ident.as_slice());
        hold.extend_from_slice(len_bin.as_slice());
        hold.extend_from_slice(self.data.as_slice());

        let mut to_return = self.amble.clone();

        to_return.extend_from_slice(Frame::encode(hold.as_slice()).as_slice());

        to_return
    }
}
