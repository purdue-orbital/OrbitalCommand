use crate::tools::{bin_to_u8, u8_to_bin};
use crate::{AMBLE,IDENT};

/// The Frame design implemented here is CCSDS SDLP which is specifically designed for use in
/// spacecraft and space bound communication
///
/// Here is the official standard: https://public.ccsds.org/Pubs/132x0b3.pdf
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
}


impl Frame {
    /// Create a new frame object given data the will be encapsulated by the frame
    pub fn new(bytes: &[u8]) -> Frame {
        Frame { version_number: 0, spacecraft_id: 0, virtual_channel_id: 0, ocf: false, master_frame_count: 0, virtual_frame_count: 0, data_status: 0, data: bytes.to_vec() }
    }

    /// Turn a string into frame segments (if any)
    pub fn from(data: Vec<String>) -> Vec<Frame>
    {
        // Create return vector
        let mut to_return = Vec::new();

        for x in data {
            to_return.push(Frame { version_number: 0, spacecraft_id: 0, virtual_channel_id: 0, ocf: false, master_frame_count: 0, virtual_frame_count: 0, data_status: 0, data: bin_to_u8(x.as_str()) });
        }

        to_return
    }

    /// This will assemble the frame header and make it ready to be transmitted
    pub fn assemble(&self) -> Vec<u8> {
        let bin = u8_to_bin(self.data.as_slice());

        let len = self.data.len() as u16;

        let len_bin = u8_to_bin(&[(len >> 8) as u8, len as u8]);

        let amble= AMBLE;
        let ident = IDENT;

        bin_to_u8(format!("{amble}{ident}{len_bin}{bin}").as_str())
    }
}
