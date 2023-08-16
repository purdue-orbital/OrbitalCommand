use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use crate::device::Device;
use crate::layer_3::icmp::{IcmpTypes, ICMPv4};
use crate::layer_3::ipv4::{Address};
use anyhow::Result;

/// This trait is what allows different services to run on internet ports
pub trait Service: Send + Sync {

    /// **MUST BE THREAD SAFE**
    ///
    /// Inbound transmissions will be passed to this function
    ///
    /// # Parameter(s):
    /// - 'inbound' - this is the raw data that is passed to it
    ///
    /// # Return
    /// - bool value of if this service should be disabled
    fn run_service(&mut self, inbound:&[u8]) -> bool;
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone)]
struct PingServer{
    device: Arc<RwLock<Device>>
}

unsafe impl Send for PingServer{

}

unsafe impl Sync for PingServer{

}

impl Service for PingServer {
    fn run_service(&mut self, inbound: &[u8]) -> bool {

        let mut packet = ICMPv4::decode(inbound).unwrap();

        std::mem::swap(&mut packet.header.source_ip_address, &mut packet.header.destination_ip_address);

        packet.message_type = IcmpTypes::EchoReply as u8;

        packet.update_checksum();

        self.device.read().unwrap().stream.send(packet.encode(false).as_slice()).unwrap();

        false
    }
}

#[derive(Clone)]
struct PingClient{
    pub packet: Arc<RwLock<Option<ICMPv4>>>,
    pub time_recv: Arc<RwLock<Option<SystemTime>>>
}

impl Service for PingClient {
    fn run_service(&mut self, inbound: &[u8]) -> bool {

        self.time_recv.write().unwrap().replace(SystemTime::now());
        self.packet.write().unwrap().replace(ICMPv4::decode(inbound).unwrap());

        // we don't want to keep listening
        true
    }
}

/// This is a struct that holds the statistics about a ping request
pub struct PingStats{

    /// Did this packet make it
    pub completed: bool,

    /// The amount of time it took to complete
    pub duration: Duration,

    /// The packet sequence
    pub seq_num: u16,

    /// The length of the packet (in bytes)
    pub length: u16,

    /// The request destination (or the response source) host ip
    pub resp_addr: Address,

    /// The number of device it interacted with to send this ping (55 means 55 router/switches)
    pub hops: u8

}


pub struct Ping<'a>{
    device: &'a mut Device,

    // counter for the number of ping packets sent
    pub seq_num: u16
}

impl<'a> Ping<'a>{
    pub fn new(device:&'a mut Device) -> Ping{
        Ping{device, seq_num: 0}
    }

    /// This will start the ping service on the network device (device will become pingable)
    pub fn enable(&mut self){
        self.device.add_listen_service_without_port(Box::from(PingServer { device: Arc::new(RwLock::new(self.device.clone())) }), 1).unwrap();
    }

    /// This will stop the ping service on the network device (device will no longer be pingable)
    pub fn disable(&mut self){
        self.device.stop_listen_service_without_port(1);
    }

    /// This will send a ping
    pub fn ping(&mut self, dest:Address) -> Result<PingStats>{
        // start listen server
        let to_clone_1 = Arc::new(RwLock::new(None));
        let to_clone_2 = Arc::new(RwLock::new(None));
        let client = Box::from(PingClient{ packet: to_clone_1, time_recv: to_clone_2});
        self.device.add_listen_service_without_port(client.clone(), 1)?;

        // creates stats to set for later
        let mut stats = PingStats{
            completed: false,
            duration: Default::default(),
            seq_num: 0,
            length: 0,
            resp_addr: Address::from_str("0.0.0.0").unwrap(),
            hops: 0,
        };

        // create packet
        self.seq_num += 1;
        let mut packet = ICMPv4::new(IcmpTypes::EchoRequest, 255, self.device.ip_addr.as_ref().unwrap().clone(), dest.clone(), 586_u32 << 16 | self.seq_num as u32, &[5,5,5]);

        // encode
        let encoded = packet.encode(false);
        let encoded_slice = encoded.as_slice();

        // send
        self.device.stream.send(encoded_slice).unwrap();

        let now = SystemTime::now();

        // wait 100 ms to allow for returns
        sleep(Duration::from_millis(100));

        // handle return
        if let Some(x) = client.as_ref().packet.write().unwrap().take(){
            // set stats
            let time = client.time_recv.read().unwrap().unwrap();
            stats.length = x.header.total_length;
            stats.hops = 255 - x.header.time_to_live;
            stats.resp_addr = dest;
            stats.seq_num = self.seq_num;
            stats.duration = time.duration_since(now).unwrap();
            stats.completed = true;
        }

        // ensure closure
        self.device.stop_listen_service_without_port(1);


        Ok(stats)
    }
}


//--------------------------------------------------------------------------------------------------

#[derive(Clone)]
/// This will provide dns records upon request
struct DNSServer{
    device: Arc<RwLock<Device>>
}

unsafe impl Send for DNSServer{

}

unsafe impl Sync for DNSServer{

}

impl Service for DNSServer {
    fn run_service(&mut self, inbound: &[u8]) -> bool {

        let mut packet = ICMPv4::decode(inbound).unwrap();

        std::mem::swap(&mut packet.header.source_ip_address, &mut packet.header.destination_ip_address);

        packet.message_type = IcmpTypes::EchoReply as u8;

        packet.update_checksum();

        self.device.read().unwrap().stream.send(packet.encode(false).as_slice()).unwrap();

        false
    }
}

#[derive(Clone)]
/// This will get DNS records from a DNS server
struct DNSClient{

}

impl Service for DNSClient {
    fn run_service(&mut self, inbound: &[u8]) -> bool {

        // self.time_recv.write().unwrap().replace(SystemTime::now());
        // self.packet.write().unwrap().replace(ICMPv4::decode(inbound).unwrap());

        // we don't want to keep listening
        true
    }
}


pub struct DNS<'a>{
    device: &'a mut Device,
}

impl<'a> DNS<'a>{
    pub fn new(device:&'a mut Device) -> DNS{
        DNS{device}
    }

    /// This will start the ping service on the network device (device will become pingable)
    pub fn enable(&mut self){
        self.device.add_listen_service_without_port(Box::from(PingServer { device: Arc::new(RwLock::new(self.device.clone())) }), 1).unwrap();
    }

    /// This will stop the ping service on the network device (device will no longer be pingable)
    pub fn disable(&mut self){
        self.device.stop_listen_service_without_port(1);
    }

    /// This will send a ping
    pub fn ping(&mut self, dest:Address) -> Result<PingStats>{
        // start listen server
        let to_clone_1 = Arc::new(RwLock::new(None));
        let to_clone_2 = Arc::new(RwLock::new(None));
        let client = Box::from(PingClient{ packet: to_clone_1, time_recv: to_clone_2});
        self.device.add_listen_service_without_port(client.clone(), 1)?;

        // creates stats to set for later
        let mut stats = PingStats{
            completed: false,
            duration: Default::default(),
            seq_num: 0,
            length: 0,
            resp_addr: Address::from_str("0.0.0.0").unwrap(),
            hops: 0,
        };

        // create packet
        let mut packet = ICMPv4::new(IcmpTypes::EchoRequest, 255, self.device.ip_addr.as_ref().unwrap().clone(), dest.clone(), 586_u32 << 16 | 0u32, &[5,5,5]);

        // encode
        let encoded = packet.encode(false);
        let encoded_slice = encoded.as_slice();

        // send
        self.device.stream.send(encoded_slice).unwrap();

        let now = SystemTime::now();

        // wait 100 ms to allow for returns
        sleep(Duration::from_millis(100));

        // handle return
        if let Some(x) = client.as_ref().packet.write().unwrap().take(){
            // set stats
            let time = client.time_recv.read().unwrap().unwrap();
            stats.length = x.header.total_length;
            stats.hops = 255 - x.header.time_to_live;
            stats.resp_addr = dest;
            stats.duration = time.duration_since(now).unwrap();
            stats.completed = true;
        }

        // ensure closure
        self.device.stop_listen_service_without_port(1);


        Ok(stats)
    }
}

