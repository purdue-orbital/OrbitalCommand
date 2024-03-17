use crate::datagrams::dns::_dns::{
    BasicDns, DnsTypes, Opcode, QueryClass, QuestionSection, ResourceType, ResponseCode,
};
use crate::layer_3::ipv4::Address;
use crate::layer_4::udp::UDPv4;

/// This holds the parent class for the different DNS packet types
mod _dns {
    use num_enum::TryFromPrimitive;

    #[derive(Copy, Clone, TryFromPrimitive)]
    #[repr(u16)]
    pub enum QueryClass {
        /// Standard internet query (Use this)
        IN = 1,

        /// CSNET query (__old__ request type specifically for the CSNET network which was shut down in 1991)
        CS = 2,

        /// Chaosnet query (this too is considered old but this class is often misused for DNS data transfers)
        CH = 3,

        /// Hesiod query apart of "Project Athena" (This is also old and outdated with questionable security practices)
        HS = 4,
    }

    #[derive(Copy, Clone, TryFromPrimitive)]
    #[repr(u16)]
    pub enum ResourceType {
        /// Standard IPv4 lookup (Domain name -> IPv4)
        A = 1,

        /// Name Server (for use of DNS zones)
        NS = 2,

        /// Canonical name. "Domain name redirect" (like how gooogle.com might redirect you back to google.com) (Domain name -> Domain name)
        CNAME = 5,

        /// Start of an authority record zone (this will hold random text like an email or serial number)
        SOA = 6,

        /// A Canonical name pointer. This will return a name and only a name. (Domain Name -> Domain Name)
        PTR = 12,

        /// Mail exchange. Domain name that handles emails for this domain (Domain name -> Email Domain Name)
        MX = 15,

        /// "Human readable text" This mostly used today for robots and unique DNS instructions (Domain Name -> Text)
        TXT = 16,

        /// IPv6 Record of domain name (Domain Name -> IPv6)
        AAAA = 28,
    }

    #[derive(Clone)]
    pub struct QuestionSection {
        /// Name of resource
        pub name: String,

        /// Record Type
        pub r_type: ResourceType,

        /// Class code
        pub class: QueryClass,
    }

    impl QuestionSection {
        pub fn encode(&self) -> Vec<u8> {
            let mut to_return = self.name.as_bytes().to_vec();

            to_return.extend_from_slice(&[
                (self.r_type as u16 >> 8) as u8,
                self.r_type as u8,
                (self.class as u16 >> 8) as u8,
                self.class as u8,
            ]);

            to_return
        }

        pub fn decode(data: &[u8]) -> QuestionSection {
            let window = data.len() - 4;

            QuestionSection {
                name: String::from_utf8(data[..window].to_vec()).unwrap(),

                r_type: ResourceType::try_from(
                    (data[window] as u16) << 8 | (data[window + 1] as u16),
                )
                .unwrap(),
                class: QueryClass::try_from(
                    (data[window + 2] as u16) << 8 | (data[window + 3] as u16),
                )
                .unwrap(),
            }
        }
    }

    #[derive(Clone)]
    pub struct ResourceRecord {
        /// Name of resource
        pub name: String,

        /// Record Type
        pub r_type: ResourceType,

        /// Class code
        pub class: QueryClass,

        /// Time-to-live
        pub ttl: u32,

        /// length of the RDATA field in bytes
        pub rd_length: u16,

        /// The data of this field (like an IP address for A and AAAA records)
        pub rdata: String,
    }

    #[derive(Clone, TryFromPrimitive)]
    #[repr(u8)]
    pub enum DnsTypes {
        /// Query request (request from client)
        Query,

        /// Response from server
        Reply,
    }

    #[derive(Clone, TryFromPrimitive)]
    #[repr(u8)]
    pub enum Opcode {
        /// Standard query
        Query,

        /// Inverse query (IP address to domain name)
        IQuery,

        /// Server status response
        Status,
    }

    #[derive(Clone, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ResponseCode {
        /// No error occurred
        NoError,

        /// Format Error
        FormError,

        /// Retrieval Error
        ServFail,

        /// Non-existent domain
        NXDomain,
    }

    #[derive(Clone)]
    pub struct BasicDns {
        /// DNS type
        pub qr: DnsTypes,

        /// Operation being/was preformed
        pub opcode: Opcode,

        /// Authoritative Answer (did the server know without needing to check)
        pub aa: bool,

        /// Was this message truncated to fit length
        pub tc: bool,

        /// Does the client want a recursive query
        pub rd: bool,

        /// Does the server support recursive requests
        pub ra: bool,

        /// The server response code
        pub rcode: ResponseCode,

        /// A request can have multiple questions
        pub questions: Vec<QuestionSection>,

        /// A response can have multiple responses
        pub responses: Vec<ResourceRecord>,
    }

    impl BasicDns {
        pub fn encode(&self) -> Vec<u8> {
            // The header is only 16 bits
            let mut to_return = vec![
                (self.clone().qr as u8) << 7
                    | (self.clone().opcode as u8) << 3
                    | (self.aa as u8) << 2
                    | (self.tc as u8) << 1
                    | (self.rd as u8),
                (self.ra as u8) << 7 | (self.clone().rcode as u8),
            ];

            for x in &self.questions {
                to_return.extend_from_slice(x.encode().as_slice())
            }

            for x in &self.responses {
                //to_return.extend_from_slice()
            }

            to_return
        }

        pub fn decode(&self, data: &[u8]) -> BasicDns {
            let mut to_return = BasicDns {
                qr: DnsTypes::try_from(data[0] >> 7).unwrap(),
                opcode: Opcode::try_from((data[0] << 1) >> 4).unwrap(),
                aa: ((data[0] >> 2) & 1) == 1,
                tc: ((data[0] >> 1) & 1) == 1,
                rd: (data[0] & 1) == 1,
                ra: ((data[1] >> 7) & 1) == 1,
                rcode: ResponseCode::try_from((data[1] << 4) >> 4).unwrap(),
                questions: vec![],
                responses: vec![],
            };

            to_return.questions.push(QuestionSection {
                name: "".to_string(),
                r_type: ResourceType::A,
                class: QueryClass::IN,
            });

            to_return
        }
    }

    pub trait AsBasicDNS {
        fn as_basic_dns(&self) -> &BasicDns;
    }
}

/// UDP variant of the DNS protocol
pub struct UdpDns {
    pub dns_parent: _dns::BasicDns,
    pub udp_parent: UDPv4,
}

impl UdpDns {
    /// Generates a new request
    pub fn new_request(
        src_addr: Address,
        src_port: u16,
        dest_addr: Address,
        dest_port: u16,
        r_type: ResourceType,
        domain: String,
    ) -> UdpDns {
        // generate question
        let question = QuestionSection {
            name: domain,
            r_type,
            class: QueryClass::IN,
        };

        UdpDns {
            dns_parent: BasicDns {
                qr: DnsTypes::Query,
                opcode: Opcode::Query,
                aa: false,
                tc: false,
                rd: true,
                ra: false,
                rcode: ResponseCode::NoError,
                questions: vec![question],
                responses: vec![],
            },

            udp_parent: UDPv4::new(src_addr, src_port, dest_addr, dest_port, &[]),
        }
    }
}

impl _dns::AsBasicDNS for UdpDns {
    fn as_basic_dns(&self) -> &BasicDns {
        &self.dns_parent
    }
}
