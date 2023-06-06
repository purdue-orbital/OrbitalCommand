use crate::device::Device;
use crate::layer_3::icmp::{IcmpTypes, ICMPv4};
use anyhow::{Error, Result};

/// This trait is what allows different services to run on internet ports
pub trait Service{

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
pub struct Ping {
    device: Device
}

impl Ping {

    /// This will create and start a ping instance. (This device will become pingable)
    pub fn new(device: &mut Device) -> Result<Ping> {
        let mut out = Ping{device: device.clone()};

        //out.enable();

        Ok(out)
    }

    /// This will start a ping server (device will become pingable
    pub fn enable(&mut self){
        let dont_care = self.device.add_listen_service_without_port(Box::from(self.clone()),1);

        if let Ok(..) = dont_care{
            dont_care.unwrap();
        }else{
            // welp ¯\_(ツ)_/¯
        }
    }

    /// This will disable the current ping server
    pub fn disable(&mut self){
        self.device.stop_listen_service_without_port(1);
    }
}

impl Service for Ping {
    fn run_service(&self, inbound: &[u8]) -> bool {

        let mut packet = ICMPv4::decode(inbound).unwrap();

        std::mem::swap(&mut packet.header.source_ip_address, &mut packet.header.destination_ip_address);

        packet.message_type = IcmpTypes::EchoReply as u8;

        packet.update_checksum();

        self.device.iface.lock().unwrap().as_mut().unwrap().send(packet.encode(false).as_slice()).unwrap();

        false
    }
}




