extern crate num;

use std::{fmt, vec};
use std::fmt::Formatter;

use anyhow::{bail, Result};
use ux::{u13, u2, u3, u4, u6};

use crate::tools::{sum_with_carries, u8_arr_to_u16_arr};

/// IPv4 IP address object
pub struct Address {
    field1: u8,
    field2: u8,
    field3: u8,
    field4: u8,
}

impl Address {
    /// Convert string to IP address
    pub fn from_str(input: &str) -> Result<Address> {
        let arr = input.split('.').collect::<Vec<&str>>();

        // Ensure at least 4 fields are present
        if arr.len() < 4 {
            bail!("Invalid IP input")
        }

        Ok(Address { field1: arr[0].parse()?, field2: arr[1].parse()?, field3: arr[2].parse()?, field4: arr[3].parse()? })
    }

    /// Create ipv4 address
    pub fn new(field1: u8, field2: u8, field3: u8, field4: u8) -> Address {
        Address { field1, field2, field3, field4 }
    }

    /// Convert address to a u32 value
    pub fn encode(&self) -> u32 {
        (self.field1 as u32) << 24 | (self.field2 as u32) << 16 | (self.field3 as u32) << 8 | (self.field4 as u32)
    }

    /// Take a u32 and return an address object
    pub fn decode(input: u32) -> Address {
        Address {
            field1: (input >> 24) as u8,
            field2: (input >> 16) as u8,
            field3: (input >> 8) as u8,
            field4: input as u8,
        }
    }
}

/// make a clean version to represent an address
impl fmt::Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.field1, self.field2, self.field3, self.field4)
    }
}


/// Flags configure data about fragmentation. If "don't fragment" and "more fragments" are set, the
/// packet will just be dropped
pub struct Flags {
    /// This value is currently not in use and can just be ignored
    reserved: bool,

    /// This is set if this packet should not be broken up
    dont_fragment: bool,

    /// This is set if the packet is too large and needs to broken up more
    more_fragment: bool,
}

/// Explicit Congestion Notification (ECN) is a much more recent addition to IPV4 and is not fully
/// supported by all protocols yet has become critical to TCP particularly in recent years. The
/// role of the ECN to act as a "control flow" of data through a medium. This is how systems are
/// able to dynamically scale the bandwidth they occupy. ECN is implemented differently protocol
/// to protocol. In the case of TCP/IP if data is having trouble being delivered, a congestion
/// marker is sent in a SYN packet.
pub struct ECN {
    enabled: bool,
    congested: bool,
}

impl ECN {
    /// Create ECN
    pub fn new(enabled: bool, congested: bool) -> ECN {
        ECN { enabled, congested }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_congested(&self) -> bool {
        self.congested
    }

    pub fn encode(&self) -> u2 {
        u2::new((2 * self.enabled as u8) + (self.congested as u8))
    }

    pub fn decode(input: u2) -> ECN {
        ECN { enabled: input > u2::new(2), congested: u32::from(input) % 2 == 1 }
    }
}

/// Assured Forwarding is the second half of the DifferServ segment. These values are ignored if the
/// router is configured for ToS or Ip Precedence only. However, almost all routers today are
/// capable of understanding these. Official Assured Forwarding numbers go as AF11 or AF33 where the
/// first number, 1 in the case of AF13, is the "class" or ip precedence number and the second
/// number is the Assured Forwarding number which are declared in this enum. The number of
/// precedence are backwards compared to IP precedence or classes section as the lower the number,
/// the higher the priority it is in queue
#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum AssuredForwarding {
    /// This doesn't exist officially as this would just mean "Use IP precedence"
    AFx0,

    /// Low Drop Probability - Highest order of priority of the specific class
    AFx1,

    /// Medium Drop Probability - Default, and preferred order of priority of the specific class
    AFx2,

    /// Highest Drop Probability - Lowest order of priority of the specific class
    AFx3,
}

/// IP Precedence is considered "old" and was originally called "Type of Service" (ToS).
/// Modern routers would still be able to understand this by itself but the up to date version is
/// DiffServ which has a second component to it for more customization of priority. It should be
/// noted that legacy routers __will not__ have any trouble handling DiffServ traffic over ToS traffic
/// as the bits that followed ToS that are now used by DiffServe were marked "reserved" and were just
/// ignored by the routers that that use ToS
#[derive(Clone, Copy, FromPrimitive, Debug)]
pub enum IPPrecedence {
    /// "Default" setting - Used for standard traffic.
    DF,

    /// Priority - Data that, if given the choice, should have priority. It's recommended
    /// this to be used for the FTP or SMB protocols for file transfers
    CS1,

    /// Immediate - This is given higher precedence in transmission. It's recommended to use
    /// this level for ssh, ping, or syslog transmissions
    CS2,

    /// Flash - Higher priority that should be given to RTSP or streaming video
    CS3,

    /// Flash-override - Named after the term when the President Of The United States deems a
    /// message of highest importance and should override all other messages no matter what they are.
    /// These should be assigned to video gaming streams or video conferencing streams.
    CS4,

    /// Critical - Highest priority given to non-networking protocols. It is important
    /// that non networking protocols, like NTP, not go past this level. Some routers will
    /// block or drop the transmission as it could be deemed malicious if you go past this level.
    /// This level should be used for NTP or Peer-to-Peer (PTP) transmissions, like SIP, H.323, etc.
    /// It is also important to know that if you give all transmissions this level of precedence,
    /// then you will see no speed gain and might see all transmissions that are CS3 and below be
    /// dropped
    CS5,

    /// **(NETWORKING ONLY)** Internet - A networking protocol only precedence that should only be
    /// given to routing protocols like OSPF, BGP, RIP, ETC.
    CS6,

    /// **(NETWORKING ONLY)** Networking - Look at yourself in the mirror. Do you really want this
    /// level? Is what you're sending so important that you are willing to risk not being able to
    /// send it at all (as routing protocols will be overriden)? Nothing by default has or should
    /// have this level or precedence unless you have some custom protocol that will launch weapons
    /// of mass destruction or cure cancer in one transmission.
    CS7,
}

#[derive(Clone, Copy, Debug)]
pub struct DifferentiatedServices {
    ip_precedence: IPPrecedence,

    assured_forwarding: AssuredForwarding,
}

impl DifferentiatedServices {
    pub fn new(ip_precedence: IPPrecedence, assured_forwarding: AssuredForwarding) -> DifferentiatedServices {
        DifferentiatedServices { ip_precedence, assured_forwarding }
    }

    pub fn encode(&self) -> u6 {
        u6::new((8 * self.ip_precedence as u8) + (2 * self.assured_forwarding as u8))
    }

    pub fn decode(input: u6) -> DifferentiatedServices {
        DifferentiatedServices {
            ip_precedence: num::FromPrimitive::from_u8(u8::from(input) / 8).unwrap(),
            assured_forwarding: num::FromPrimitive::from_u8((u8::from(input) % 8) / 2).unwrap(),
        }
    }

    pub fn get_ip_precedence(&self) -> IPPrecedence {
        self.ip_precedence
    }

    pub fn get_assured_forwarding(&self) -> AssuredForwarding {
        self.assured_forwarding
    }
}

/// IPV4 datagram
pub struct IPV4 {
    version: u4,
    internet_header_length: u4,
    pub differentiated_services_code_point: u6,
    pub explicit_congestion_notification: u2,
    total_length: u16,
    pub identification: u16,
    pub flags: u3,
    pub fragment_offset: u13,
    pub time_to_live: u8,
    pub protocol: u8,
    header_checksum: u16,
    pub source_ip_address: u32,
    pub destination_ip_address: u32,
    pub option: Vec<u8>,
    data: Vec<u8>,
}

/// Default implementation of IPV4
impl Default for IPV4 {
    fn default() -> Self {
        IPV4 {
            version: u4::new(4),
            internet_header_length: Default::default(),
            differentiated_services_code_point: Default::default(),
            explicit_congestion_notification: Default::default(),
            total_length: 0,
            identification: 0,
            flags: Default::default(),
            fragment_offset: Default::default(),
            time_to_live: 64,
            protocol: 0,
            header_checksum: 0,
            source_ip_address: 0,
            destination_ip_address: 0,
            option: vec![],
            data: vec![],
        }
    }
}

/// Functions for IPV4 to use
impl IPV4 {
    /// Calculate checksum
    pub fn calc_checksum(&self) -> u16 {
        // get encoded header only
        let mut arr = self.encode(true);

        // set checksum values to 0
        arr[10] = 0;
        arr[11] = 0;

        // Convert array to an array of u16s
        let to_sum = u8_arr_to_u16_arr(arr.as_slice());

        // return one's complement of the sum with carries
        !sum_with_carries(to_sum.as_slice())
    }

    /// verify header with checksum
    pub fn verify(&self) -> bool {
        // get encoded header only
        let arr = self.encode(true);

        // Convert array to an array of u16s
        let to_sum = u8_arr_to_u16_arr(arr.as_slice());

        // evaluate if the sum is equal to u16::MAX
        sum_with_carries(to_sum.as_slice()) == 65535
    }

    #[warn(clippy::too_many_arguments)]
    pub fn new(
        data: &[u8],
        options: &[u8],
        differentiated_services: &DifferentiatedServices,
        ecn: ECN,
        identification: u16,
        fragment_offset: u13,
        time_to_live: u8,
        protocol: u8,
        source_ip_address: u32,
        destination_ip_address: u32,
    ) -> IPV4 {
        // Create ipv4
        let mut to_return = IPV4::default();

        // convert options to a vec
        let mut options_vec = options.to_vec();

        // pad out the options to a multiple of 32 (if needed)
        while (options_vec.len() * 8) % 32 != 0 {
            options_vec.push(0);
        }

        //   (For ipv4, this is always 20)
        //              V
        // IHL = (default_header_length + options_length) / 4
        let ihl = (20 + options_vec.len()) / 4;

        // set ipv4 main header settings
        to_return.internet_header_length = u4::new(ihl as u8);
        to_return.differentiated_services_code_point = u6::new(u8::from(differentiated_services.encode()));
        to_return.explicit_congestion_notification = u2::new(u8::from(ecn.encode()));


        to_return.identification = identification;
        to_return.fragment_offset = fragment_offset;
        to_return.time_to_live = time_to_live;

        to_return.protocol = protocol;

        to_return.source_ip_address = source_ip_address;
        to_return.destination_ip_address = destination_ip_address;

        to_return.option = options_vec;
        to_return.set_data(data);

        to_return.header_checksum = to_return.calc_checksum();

        //return ipv4
        to_return
    }

    /// Update the packet checksum
    pub fn update_checksum(&mut self) {
        self.header_checksum = self.calc_checksum();
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.total_length = (u16::from(self.internet_header_length) * 4) + data.len() as u16;
        self.data = data.to_vec();
    }


    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Encodes the values set in the IPV4 header into bytes (for transmission)
    pub fn encode(&self, ignore_data: bool) -> Vec<u8> {

        // Convert each section into a u8
        let mut to_return = vec! {
            (u8::from(self.version) << 4) | (u8::from(self.internet_header_length)),
            u8::from(self.differentiated_services_code_point) << 2 | u8::from(self.explicit_congestion_notification),
            (self.total_length >> 8) as u8,
            self.total_length as u8,
            (self.identification >> 8) as u8,
            self.identification as u8,
            u8::from(self.flags) << 5 | (u16::from(self.fragment_offset) >> 8) as u8,
            u16::from(self.fragment_offset) as u8,
            self.time_to_live,
            self.protocol,
            (self.header_checksum >> 8) as u8,
            self.header_checksum as u8,
            (self.source_ip_address >> 24) as u8,
            (self.source_ip_address >> 16) as u8,
            (self.source_ip_address >> 8) as u8,
            self.source_ip_address as u8,
            (self.destination_ip_address >> 24) as u8,
            (self.destination_ip_address >> 16) as u8,
            (self.destination_ip_address >> 8) as u8,
            self.destination_ip_address as u8,
        };

        to_return.extend_from_slice(self.option.as_slice());

        if !ignore_data {
            to_return.extend_from_slice(self.data.as_slice());
        }

        to_return
    }

    pub fn decode(arr: &[u8]) -> IPV4 {
        let total_length = ((arr[2] as u16) << 8) | (arr[3] as u16);
        let internet_header_length = u4::new((arr[0] << 4) >> 4); // Who said hacks are bad?

        // Scary looking bit manipulations
        IPV4 {
            version: u4::new(4),
            internet_header_length,
            differentiated_services_code_point: u6::new(arr[1] >> 2),
            explicit_congestion_notification: u2::new((arr[1] << 6) >> 6), // Ha ha more hacks!
            total_length,
            identification: ((arr[4] as u16) << 8) | (arr[5] as u16),
            flags: u3::new(arr[6] >> 5),
            fragment_offset: u13::new(((arr[6] as u16) << 8) | (arr[7] as u16)),
            time_to_live: arr[8],
            protocol: arr[9],
            header_checksum: (arr[10] as u16) << 8 | (arr[11] as u16),
            source_ip_address: (arr[12] as u32) << 24 | (arr[13] as u32) << 16 | (arr[14] as u32) << 8 | (arr[15] as u32),
            destination_ip_address: (arr[16] as u32) << 24 | (arr[17] as u32) << 16 | (arr[18] as u32) << 8 | (arr[19] as u32),
            option: arr[20..(u8::from(internet_header_length) * 4) as usize].to_vec(),
            data: arr[(u8::from(internet_header_length) * 4) as usize..total_length as usize].to_vec(),
        }
    }
}