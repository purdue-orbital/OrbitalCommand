

use anyhow::Result;
use soapysdr::{Device};

// Radio Values
#[derive(Clone)]
pub struct Radio {
    device: Option<Device>,
    // Attached radio instance
    is_connected: bool,
}

impl Radio {
    /// New Radio Instance
    /// This will attempt to connect to a radio connected to the system
    pub fn new() -> Result<Radio> {
        // Initialize values of a radio
        let mut new_radio = Radio { device: None, is_connected: false };

        // Set that we are looking for lime devices
        let mut args = soapysdr::Args::new();

        // For some reason this is platform dependent
        args.set("driver", "lime");

        // get list of radios
        let err = Device::new(args);

        // if we get the radio properly, set the radio data
        if !err.is_err() {
            // if we find a radio and connect to it
            new_radio.device = Some(err.unwrap());
            new_radio.is_connected = true;
        }

        Ok(new_radio)
    }

    /// Return bool value of if the radio is connected to the system
    pub fn is_connected(&self) -> bool {
        return self.is_connected;
    }

    /// Get Radio
    /// This will get an already established radio instance so you don't have to try to reconnect
    pub fn get_radio(&self) -> &Device {
        self.device.as_ref().expect("Get Radio Instance")
    }
}
