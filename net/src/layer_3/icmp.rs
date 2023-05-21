use ux::u13;
use crate::layer_3::ipv4::{Address, DifferentiatedServices, ECN, IPPrecedence, IPV4};
use crate::layer_3::ipv4::AssuredForwarding::{AFx0, AFx2};
use crate::layer_3::ipv4::IPPrecedence::DF;
use crate::tools::{sum_with_carries, u8_arr_to_u16_arr};

#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum IcmpTypes {
    /// Response message
    EchoReply,

    Unassigned1,
    Unassigned2,

    DestinationUnreachable,

    /// Deprecated
    SourceQuench,

    RedirectMessage,

    Unassigned6,
    Unassigned7,

    /// Request echo (standard)
    EchoRequest,

    RouterAdvertisement,
    RouterSolicitation,

    /// Time has exceeded
    TimeExceeded,

    BadIPHeader,
    Timestamp,
    TimestampReply,

    /// Deprecated
    InformationRequest,

    /// Deprecated
    InformationReply,

    /// Deprecated
    AddressMaskRequest,

    /// Deprecated
    AddressMaskReply,

    Unassigned19,
    Unassigned20,
    Unassigned21,
    Unassigned22,
    Unassigned23,
    Unassigned24,
    Unassigned25,
    Unassigned26,
    Unassigned27,
    Unassigned28,
    Unassigned29,

    /// Deprecated
    Traceroute,

    Unassigned31,
    Unassigned32,
    Unassigned33,
    Unassigned34,
    Unassigned35,
    Unassigned36,
    Unassigned37,
    Unassigned38,
    Unassigned39,
    Unassigned40,
    Unassigned41,

    ExtendedEchoRequest,
    ExtendedEchoReply,
}


/// Internet Control Message Protocol (ICMP) is also sometimes called a "ping" packet. This is the
/// protocol that is used to help troubleshoot networks and make sure everything is working
pub struct ICMPv4{
    /// The header section of this packet
    pub header:IPV4,

    pub message_type:u8,
    code:u8,
    checksum:u16,

    /// this isn't uniform what is in here and often acts like the "data" section of the packet
    rest_of_header:u32,

    data:Vec<u8>
}

impl ICMPv4{

    pub fn new(icmp_type:IcmpTypes,time_to_live:u8,source_ip:Address, dest_ip:Address, rest_of_header:u32, data:&[u8]) -> ICMPv4{

        // create header
        let header = IPV4::new(
            &[],
            &[],
            &DifferentiatedServices::new(DF, AFx0), // ICMP really isn't important
            ECN::new(false,false), // ICMP doesn't use ECN
            9210,
            u13::new(0),
            time_to_live,
            1,
            source_ip.encode(),
            dest_ip.encode()
        );

        let mut out = ICMPv4{
            header,
            message_type: icmp_type as u8,
            code: 0,
            checksum: 0,
            rest_of_header,
            data:data.to_vec(),
        };

        // calculate checksum and return
        out.checksum = out.calc_checksum();

        out
    }

    pub fn calc_checksum(&mut self) -> u16 {
        // encode
        let mut encoded = self.encode(true);

        // set checksum bytes to 0
        encoded[2] = 0;
        encoded[3] = 0;

        // convert to u16s
        let arr = u8_arr_to_u16_arr(encoded.as_slice());

        // calculate and return the one's complement
        !sum_with_carries(arr.as_slice())
    }

    pub fn update_checksum(&mut self) {
        self.checksum = self.calc_checksum();
    }

    /// Verify the packet with the checksum
    pub fn verify(&mut self) -> bool{
        // encode
        let encoded = self.encode(true);

        // convert to u16s
        let arr = u8_arr_to_u16_arr(encoded.as_slice());

        // calculate and return the one's complement
        sum_with_carries(arr.as_slice()) == 65535
    }

    // encode icmp
    pub fn encode(&mut self, ignore_ipv4_header:bool) -> Vec<u8>{

        let mut to_return = vec![
            self.message_type,
            self.code,
            (self.checksum >> 8) as u8,
            self.checksum as u8,
            (self.rest_of_header >> 24) as u8,
            (self.rest_of_header >> 16) as u8,
            (self.rest_of_header >> 8) as u8,
            self.rest_of_header as u8,
        ];

        to_return.extend_from_slice(self.data.as_slice());

        if !ignore_ipv4_header {
            self.header.set_data(to_return.as_slice());

            self.header.update_checksum();

            to_return = self.header.encode(false);
        }

        to_return
    }

    /// Decode an array of u8 into a ICMP packet
    pub fn decode(input:&[u8]) -> ICMPv4{
        let ipv4 = IPV4::decode(input);
        let data:Vec<u8> = ipv4.get_data();

        ICMPv4{
            header: ipv4,
            message_type: data[0],
            code: data[1],
            checksum: (data[2] as u16) << 8 | data[3] as u16,
            rest_of_header: (data[4] as u32) << 24 | (data[5] as u32) << 16 | (data[6] as u32) << 8 | data[7] as u32,
            data: data[8..].to_vec(),
        }
    }
}
