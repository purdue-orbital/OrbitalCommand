#[macro_use]
extern crate num_derive;
extern crate tun_tap;
extern crate radio;


use tun_tap::{Iface, Mode};
use radio::RadioStream;


/// Get dependent crates
mod tools;
pub mod layer_3{
    pub mod ipv4;
    pub mod icmp;
}

/// These are possible interfaces that could be present on a device
#[derive(PartialEq)]
pub enum Interface {
    /// Use the connected Software Defined Radio
    SDR,

    /// Use what ever the system prefers (like wlan or ethernet)
    System
}

/// This is a instance of a device that can be connected to
pub struct Device {
    pub interface: Interface,
    pub name: String,

    pub iface: Option<Iface>,
    pub radio: Option<RadioStream>,
}

/// List all device that could be connected to this computer
pub fn list_devices() -> Vec<Device>{
    let mut list = Vec::new();

    // create system device
    let sys = Iface::new("", Mode::Tap);

    // add device to list if setup worked
    if let Ok(..) = sys{
        list.push(Device{
            interface: Interface::System,
            name: sys.as_ref().unwrap().name().to_string(),

            iface: Some(sys.unwrap()),
            radio: None
        })
    }

    // create radio device
    let sdr = RadioStream::new();

    if let Ok(..) = sdr{
        list.push(Device{
            interface: Interface::SDR,
            name: "SDR".to_string(),

            iface: None,
            radio: Some(sdr.unwrap())
        })
    }

    list
}

/// Main struct
pub struct Net{
    device: Device
}

/// Main impl
impl Net {

    // Initialize a Net object
    pub fn new(device:Device) -> Net{
        Net{device}
    }

    //
    pub fn listen(){

    }
}