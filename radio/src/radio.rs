use anyhow::Result;
use soapysdr::{Device, ErrorCode};

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

        args.set("driver", "bladerf");

        // get a list of radios
        let err = Device::new(args);

        // if we get the radio properly, set the radio data
        if let Ok(x) = err {
            // if we find a radio and connect to it
            new_radio.device = Some(x);
            new_radio.is_connected = true;
        }

        Ok(new_radio)
    }

    /// Return bool value of if the radio is connected to the system
    pub fn is_connected(&self) -> bool { self.is_connected }

    /// Get Radio
    /// This will get an already established radio instance so you don't have to try to reconnect
    pub fn get_radio(&self) -> Result<&Device, soapysdr::Error> {
        if let Some(x) = self.device.as_ref() {
            Ok(x)
        } else {
            Err(
                soapysdr::Error {
                    code: ErrorCode::Other,
                    message: "Unable to fetch radio!".to_string(),
                }
            )
        }
    }
}
