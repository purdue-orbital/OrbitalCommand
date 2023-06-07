use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use crate::device::Device;
use crate::layer_3::icmp::{IcmpTypes, ICMPv4};
use anyhow::{Error, Result};

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
    fn run_service(&self, inbound:&[u8]) -> bool;
}

#[derive(Clone)]
struct PingServer{
    device: Arc<Mutex<Device>>
}

impl Service for PingServer {
    fn run_service(&self, inbound: &[u8]) -> bool {

        let mut packet = ICMPv4::decode(inbound).unwrap();

        std::mem::swap(&mut packet.header.source_ip_address, &mut packet.header.destination_ip_address);

        packet.message_type = IcmpTypes::EchoReply as u8;

        packet.update_checksum();

        self.device.lock().unwrap().iface.read().unwrap().as_ref().unwrap().send(packet.encode(false).as_slice()).unwrap();

        false
    }
}

pub struct Ping<'a>{
    device: &'a mut Device,
}

impl<'a> Ping<'a>{
    pub fn new(device:&'a mut Device) -> Ping{
        Ping{device}
    }

    /// This will start the ping service on the network device (device will become pingable)
    pub fn enable(&mut self){
        self.device.add_listen_service_without_port(Box::from(PingServer { device: Arc::new(Mutex::new(self.device.clone())) }), 1).unwrap();
    }

    /// This will stop the ping service on the network device (device will no longer be pingable)
    pub fn disable(&mut self){
        self.device.stop_listen_service_without_port(1);
    }
}




