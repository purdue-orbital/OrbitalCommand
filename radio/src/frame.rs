//! This file contains the Frame object struct and implementation. Frame is the lowest OSI level
//! building block which is best handed and managed by the system itself. This particular frame
//! header implements CCSDS SDLP standard which is the standard frame header for space communication


use bytes::{Buf, Bytes};
use lazy_static::lazy_static;
use rustdsp::ecc::wtf_ecc::WtfECC;

use crate::{AMBLE, IDENT};
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
        let mut encoder = WtfECC::default();
        let mut output = encoder.encode(Bytes::copy_from_slice(bin));
        let len = output.remaining();

        output.copy_to_bytes(len).to_vec()
    }

    fn decode(bin: &[u8]) -> Vec<u8> {
        let mut decoder = WtfECC::default();

        let mut bytes = Bytes::copy_from_slice(bin);

        decoder.decode(&mut bytes)
            .copy_to_bytes(bin.len() / 3)
            .to_vec()
    }

    /// Turn a string into frame segments (if any)
    pub fn from(data: &[u8]) -> Frame
    {
        let mut new_frame = Frame::new(&[]);

        new_frame.has_ident = false;
        new_frame.is_complete = false;

        let ident_length_bytes = new_frame.ident.len();

        let ident_decoded = Frame::decode(&data[..12]);

        if ident_decoded.len() >= ident_length_bytes && ident_decoded == IDENT_VEC.as_slice() {
            new_frame.has_ident = true;

            if data.len() / 3 >= ident_length_bytes + 3 {
                let len_bin = Frame::decode(&data[12..18]);

                new_frame.len = (len_bin[0] as u16) << 8 | len_bin[1] as u16;

                if data.len() / 3 == ident_length_bytes + 2 + new_frame.len as usize {
                    new_frame.is_complete = true;

                    let data_bin = Frame::decode(&data[18..]);

                    new_frame.data = data_bin.to_vec();
                }
            }
        }


        new_frame
    }

    /// This will assemble the frame header and make it ready to be transmitted
    pub fn assemble(&self) -> Vec<u8> {
        let len_bin = &[(self.len >> 8) as u8, self.len as u8];

        let mut to_return = self.amble.clone();

        to_return.extend_from_slice(Frame::encode(self.ident.as_slice()).as_slice());
        to_return.extend_from_slice(Frame::encode(len_bin.as_slice()).as_slice());
        to_return.extend_from_slice(Frame::encode(self.data.as_slice()).as_slice());

        to_return
    }
}
