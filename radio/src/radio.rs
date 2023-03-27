use soapysdr::{Device, Error};
use std::any::type_name;

// Radio Values
#[derive(Clone)]
pub struct Radio {
    device: Option<Device>, // Attached radio instance
}

impl Radio {
    /// New Radio Instance
    /// This will attempt to connect to a radio connected to the system
    pub fn new() -> Result<Radio, Error> {
        // Initialize values of a radio
        let mut new_radio = Radio { device: None };

        // Set that we are looking for lime devices
        let mut args = soapysdr::Args::new();
        args.set("device", "lime");

        // get list of radios
        let err = soapysdr::Device::new(args)?;

        // if we find a radio and connect to it, stop looping
        new_radio.device = Some(err);
        Ok(new_radio)
    }

    /// Get Radio
    /// This will get an already established radio instance so you don't have to try to reconnect
    pub fn get_radio(&self) -> &Device {
        self.device.as_ref().expect("Get Radio Instance")
    }
}
