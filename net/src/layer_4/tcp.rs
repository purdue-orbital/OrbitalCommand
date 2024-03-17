use anyhow::{Error, Result};
use ux::{u13, u4};

use crate::layer_3::ipv4::{
    Address, AssuredForwarding, DifferentiatedServices, IPPrecedence, ECN, IPV4,
};
use crate::tools::{sum_with_carries, u8_arr_to_u16_arr};

/// These are flags to specify the specifics of this packet
pub enum TcpFlags {
    Cwr,
    Ece,
    Urg,
    Ack,
    Psh,
    Rst,
    Syn,
    Fin,
}

/// IPv4 version of the TCP protocol. TCP is an OSI Layer 4 encapsulation that is stateful
pub struct TCPv4 {
    /// IPv4 header
    pub ipv4: IPV4,

    /// Port number that the packet originated from
    pub src_port: u16,

    /// The port number that the packet is to be sent to
    pub dst_port: u16,

    /// Sequence number
    pub seq_num: u32,

    /// Acknowledgment number
    pub ack_num: u32,

    /// Tells where the data is (as header can have a dynamic length)
    pub data_offset: u4,

    // Various TCP flags
    pub cwr: bool,
    pub ece: bool,
    pub urg: bool,
    pub ack: bool,
    pub psh: bool,
    pub rst: bool,
    pub syn: bool,
    pub fin: bool,

    /// Length of data the receiver is willing to get in return
    pub window_size: u16,

    /// The checksum to verify the integrity of the header and data
    checksum: u16,

    /// offset of the sequence number to the last urgent data byte
    urgent_pointer: u16,

    /// options values
    pub options: Vec<u8>,

    /// Data that is encapsulated in the packet
    pub data: Vec<u8>,
}

impl TCPv4 {
    /// Create / initialize a tcp packet
    pub fn new(
        src_addr: Address,
        src_port: u16,
        dest_addr: Address,
        dst_port: u16,
        data: &[u8],
        options: &[u8],
        flags: &[TcpFlags],
        seq_num: u32,
        ack_num: u32,
        window_size: u16,
        urgent_pointer: u16,
    ) -> TCPv4 {
        // pad out options
        let mut options = options.to_vec();
        while options.len() % 4 != 0 {
            options.push(0);
        }

        // calculate offset
        let offset = u4::new((20 + options.len() as u8) / 4);

        // Create packet
        let mut to_return = TCPv4 {
            // create ipv4
            ipv4: IPV4::new(
                // This is set later (tcp is the data)
                &[],
                &[],
                &DifferentiatedServices::new(IPPrecedence::DF, AssuredForwarding::AFx2),
                ECN::new(false, false),
                65461,
                u13::new(0),
                64,
                // always will be 6
                6,
                src_addr.encode(),
                dest_addr.encode(),
            ),

            // set TCP data
            src_port,
            dst_port,

            seq_num,
            ack_num,

            data_offset: offset,

            cwr: false,
            ece: false,
            urg: false,
            ack: false,
            psh: false,
            rst: false,
            syn: false,
            fin: false,

            checksum: 0,
            urgent_pointer,
            options,
            data: data.to_vec(),
            window_size,
        };

        // set flags
        for x in flags {
            match x {
                TcpFlags::Cwr => to_return.cwr = true,
                TcpFlags::Ece => to_return.ece = true,
                TcpFlags::Urg => to_return.urg = true,
                TcpFlags::Ack => to_return.ack = true,
                TcpFlags::Psh => to_return.psh = true,
                TcpFlags::Rst => to_return.rst = true,
                TcpFlags::Syn => to_return.syn = true,
                TcpFlags::Fin => to_return.fin = true,
            }
        }

        to_return.update_checksum();

        to_return
    }

    /// returns if this packet passes the checksum
    pub fn verify(&mut self) -> bool {
        let length: u16 = (20 + self.options.len() + self.data.len()) as u16;

        // create IPv4 pseudo header
        let mut pseudo_header = vec![
            (self.ipv4.source_ip_address >> 16) as u16,
            self.ipv4.source_ip_address as u16,
            (self.ipv4.destination_ip_address >> 16) as u16,
            self.ipv4.destination_ip_address as u16,
            self.ipv4.protocol as u16,
            length,
            self.src_port,
            self.dst_port,
            (self.seq_num >> 16) as u16,
            self.seq_num as u16,
            (self.ack_num >> 16) as u16,
            self.ack_num as u16,
            (u16::from(self.data_offset) << 12)
                | (self.cwr as u16) << 7
                | (self.ece as u16) << 6
                | (self.urg as u16) << 5
                | (self.ack as u16) << 4
                | (self.psh as u16) << 3
                | (self.rst as u16) << 2
                | (self.syn as u16) << 1
                | self.fin as u16,
            self.window_size,
            self.checksum,
            self.urgent_pointer,
        ];

        // add data to pseudo header
        pseudo_header.extend_from_slice(u8_arr_to_u16_arr(self.options.as_slice()).as_slice());
        pseudo_header.extend_from_slice(u8_arr_to_u16_arr(self.data.as_slice()).as_slice());

        // verify packet's integrity
        sum_with_carries(pseudo_header.as_slice()) == 65535 && self.ipv4.verify()
    }

    /// This will calculate the checksum for this packet
    pub fn calc_checksum(&mut self) -> u16 {
        let length: u16 = (20 + self.options.len() + self.data.len()) as u16;

        // create IPv4 pseudo header
        let mut pseudo_header = vec![
            (self.ipv4.source_ip_address >> 16) as u16,
            self.ipv4.source_ip_address as u16,
            (self.ipv4.destination_ip_address >> 16) as u16,
            self.ipv4.destination_ip_address as u16,
            self.ipv4.protocol as u16,
            length,
            self.src_port,
            self.dst_port,
            (self.seq_num >> 16) as u16,
            self.seq_num as u16,
            (self.ack_num >> 16) as u16,
            self.ack_num as u16,
            (u16::from(self.data_offset) << 12)
                | (self.cwr as u16) << 7
                | (self.ece as u16) << 6
                | (self.urg as u16) << 5
                | (self.ack as u16) << 4
                | (self.psh as u16) << 3
                | (self.rst as u16) << 2
                | (self.syn as u16) << 1
                | self.fin as u16,
            self.window_size,
            0,
            self.urgent_pointer,
        ];

        // add data to pseudo header
        pseudo_header.extend_from_slice(u8_arr_to_u16_arr(self.options.as_slice()).as_slice());
        pseudo_header.extend_from_slice(u8_arr_to_u16_arr(self.data.as_slice()).as_slice());

        // Return one's complement of sum
        !sum_with_carries(pseudo_header.as_slice())
    }

    /// update the checksum of this packet (Needs to be done after any updates)
    pub fn update_checksum(&mut self) {
        self.ipv4.update_checksum();
        self.checksum = self.calc_checksum();
    }

    /// encode the data into a vector of u8s
    pub fn encode(&mut self, ignore_ipv4: bool) -> Vec<u8> {
        // Encode this packet
        let mut arr = vec![
            (self.src_port >> 8) as u8,
            self.src_port as u8,
            (self.dst_port >> 8) as u8,
            self.dst_port as u8,
            (self.seq_num >> 24) as u8,
            (self.seq_num >> 16) as u8,
            (self.seq_num >> 8) as u8,
            self.seq_num as u8,
            (self.ack_num >> 24) as u8,
            (self.ack_num >> 16) as u8,
            (self.ack_num >> 8) as u8,
            self.ack_num as u8,
            (u8::from(self.data_offset) << 4),
            (self.cwr as u8) << 7
                | (self.ece as u8) << 6
                | (self.urg as u8) << 5
                | (self.ack as u8) << 4
                | (self.psh as u8) << 3
                | (self.rst as u8) << 2
                | (self.syn as u8) << 1
                | self.fin as u8,
            (self.window_size >> 8) as u8,
            self.window_size as u8,
            (self.checksum >> 8) as u8,
            self.checksum as u8,
            (self.urgent_pointer >> 8) as u8,
            self.urgent_pointer as u8,
        ];

        arr.extend_from_slice(self.options.as_slice());
        arr.extend_from_slice(self.data.as_slice());

        // Ensure we also want the ipv4 header as well
        if !ignore_ipv4 {
            self.ipv4.set_data(arr.as_slice());

            self.ipv4.update_checksum();

            arr = self.ipv4.encode(false);
        }

        // return
        arr
    }

    /// decode an array of u8s into an tcp packet
    pub fn decode(arr: &[u8]) -> Result<TCPv4> {
        // decode to ipv4
        let ipv4 = IPV4::decode(arr)?;

        // get ipv4 data
        let data = ipv4.get_data();

        // ensure integrity
        if data.len() <= 20 {
            return Err(Error::msg("Packet too short for TCP!"));
        }

        let data_offset = u4::new(data[12] >> 4);

        Ok(TCPv4 {
            ipv4,
            src_port: ((data[0] as u16) << 8) | data[1] as u16,
            dst_port: ((data[2] as u16) << 8) | data[3] as u16,
            seq_num: ((data[4] as u32) << 24)
                | ((data[5] as u32) << 16)
                | ((data[6] as u32) << 8)
                | data[7] as u32,
            ack_num: ((data[8] as u32) << 24)
                | ((data[9] as u32) << 16)
                | ((data[10] as u32) << 8)
                | data[11] as u32,
            data_offset,
            cwr: data[13] & 1 == 1,
            ece: (data[13] >> 1) & 1 == 1,
            urg: (data[13] >> 2) & 1 == 1,
            ack: (data[13] >> 3) & 1 == 1,
            psh: (data[13] >> 4) & 1 == 1,
            rst: (data[13] >> 5) & 1 == 1,
            syn: (data[13] >> 6) & 1 == 1,
            fin: (data[13] >> 7) & 1 == 1,
            window_size: ((data[14] as u16) << 8) | data[15] as u16,
            checksum: ((data[16] as u16) << 8) | data[17] as u16,
            urgent_pointer: ((data[18] as u16) << 8) | data[19] as u16,
            options: data[20..(u8::from(data_offset) as usize * 4)].to_vec(),
            data: data[(u8::from(data_offset) as usize * 4)..].to_vec(),
        })
    }
}
