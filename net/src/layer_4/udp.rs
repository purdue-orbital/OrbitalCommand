use ux::u13;

use crate::layer_3::ipv4::{Address, AssuredForwarding, DifferentiatedServices, ECN, IPPrecedence, IPV4};
use crate::tools::{sum_with_carries, u8_arr_to_u16_arr};

/// IPv4 version of the UDP protocol. UDP is an OSI Layer 4 encapsulation that is connectionless or
/// stateless
pub struct UDPv4 {
    /// IPv4 header
    pub ipv4: IPV4,

    /// Port number that the packet originated from
    pub src_port: u16,

    /// The port number that the packet is to be sent to
    pub dst_port: u16,

    /// Length of the UDP header + UDP data
    length: u16,

    /// The checksum to verify the integrity of the header and data
    checksum: u16,

    /// Data that is encapsulated in the packet
    pub data: Vec<u8>,
}

impl UDPv4 {
    /// Create / initialize a UDP packet
    pub fn new(src_addr: Address, src_port: u16, dest_addr: Address, dst_port: u16, data: &[u8]) -> UDPv4 {

        // calculate length of packet
        let length = (data.len() + 8) as u16;

        // Create packet
        let mut to_return = UDPv4 {
            ipv4: IPV4::new(
                // This is set latter (udp is the data)
                &[],
                &[],
                &DifferentiatedServices::new(IPPrecedence::DF, AssuredForwarding::AFx2),
                ECN::new(false, false),
                65461,
                u13::new(0),
                64,

                // always will be 17
                17,
                src_addr.encode(),
                dest_addr.encode(),
            ),
            src_port,
            dst_port,
            length,
            checksum: 0,
            data: data.to_vec(),
        };

        to_return.update_checksum();

        to_return
    }

    /// returns if this packet passes the checksum
    pub fn verify(&mut self) -> bool {

        // create IPv4 pseudo header
        let mut pseudo_header = vec![
            (self.ipv4.source_ip_address >> 16) as u16,
            self.ipv4.source_ip_address as u16,
            (self.ipv4.destination_ip_address >> 16) as u16,
            self.ipv4.destination_ip_address as u16,
            self.ipv4.protocol as u16,
            self.length,
            self.src_port,
            self.dst_port,
            self.length,
            self.checksum,
        ];

        // add data to pseudo header
        pseudo_header.extend_from_slice(u8_arr_to_u16_arr(self.data.as_slice()).as_slice());

        // verify packet's integrity
        sum_with_carries(pseudo_header.as_slice()) == 65535 && self.ipv4.verify()
    }

    /// This will calculate the checksum for this packet
    pub fn calc_checksum(&mut self) -> u16 {

        // create IPv4 pseudo header
        let mut pseudo_header = vec![
            (self.ipv4.source_ip_address >> 16) as u16,
            self.ipv4.source_ip_address as u16,
            (self.ipv4.destination_ip_address >> 16) as u16,
            self.ipv4.destination_ip_address as u16,
            self.ipv4.protocol as u16,
            self.length,
            self.src_port,
            self.dst_port,
            self.length,
            0,
        ];

        // add data to pseudo header
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
            (self.length >> 8) as u8,
            self.length as u8,
            (self.checksum >> 8) as u8,
            self.checksum as u8,
        ];

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

    /// decode an array of u8s into an UDP packet
    pub fn decode(arr: &[u8]) -> UDPv4 {
        let ipv4 = IPV4::decode(arr);
        let data = ipv4.get_data();

        UDPv4 {
            ipv4,
            src_port: ((data[0] as u16) << 8) | data[1] as u16,
            dst_port: ((data[2] as u16) << 8) | data[3] as u16,
            length: ((data[4] as u16) << 8) | data[5] as u16,
            checksum: ((data[6] as u16) << 8) | data[7] as u16,
            data: data[8..].to_vec(),
        }
    }
}