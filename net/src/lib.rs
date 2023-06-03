#[macro_use]
extern crate num_derive;
extern crate radio;
extern crate tun_tap;


use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use tun_tap::{Iface, Mode};

use radio::RadioStream;

/// Get dependent crates
mod tools;

pub mod layer_3 {
    pub mod ipv4;
    pub mod icmp;
}

/// These are possible interfaces that could be present on a device
#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub enum Interface {
    /// Use the connected Software Defined Radio
    SDR,

    /// Use what ever the system prefers (like wlan or ethernet)
    System,
}

/// This is a instance of a device that can be connected to
#[derive(Clone)]
pub struct Device {
    pub interface: Interface,
    pub name: String,

    pub iface: Arc<Mutex<Option<Iface>>>,
    pub radio: Arc<Mutex<Option<RadioStream>>>,
}

impl Device {
    pub fn set_ip(&self, gateway: &str, ip: &str) {
        let command = format!("ip addr add dev {} local {gateway} remote {ip}", self.name);

        run_commands(command.as_str());
    }
}

fn run_commands(command: &str) -> String {

    // breakup commands
    let brokenup = command.split('\n').map(|b| { b.split(' ').collect::<Vec<&str>>() }).collect::<Vec<Vec<&str>>>();

    let mut to_return = String::new();

    // Run command
    for x in brokenup {
        if x.len() > 1 {
            let op = Command::new(x[0]).args(&x[1..]).output().unwrap();

            let output = String::from_utf8_lossy(&op.stdout);

            to_return.push_str(&output);
        }
    }

    to_return
}

/// Return all network ports on this device
fn get_ports() -> Vec<String> {
    let mut to_return = Vec::new();

    // This command will ist all interfaces
    let com = "ip link show up";
    let output = run_commands(com);

    // Go through each line and extract the device name
    for port in output.split('\n').collect::<Vec<&str>>() {

        // Check if open device and line
        if port.contains("LOWER_UP") && port.contains("BROADCAST") {

            // Add name to array
            to_return.push(port.split(": ").collect::<Vec<&str>>()[1].to_string());
        }
    }

    to_return
}

/// List all device that could be connected to this computer
pub fn list_devices() -> Vec<Device> {
    let mut list = Vec::new();


    // loop through ports
    for x in get_ports() {

        // create system device
        let sys = Iface::without_packet_info("tun%d", Mode::Tun).unwrap();

        // get device name
        let name = sys.name();

        // startup instance command
        let startup = format!(
            "ip link set dev {name} up"
        );

        // run startup
        run_commands(startup.as_str());

        // Setup port to accept and send traffic
        let command = format!("iptables -t filter -I FORWARD -i {name} -o {x} -j ACCEPT
iptables -t filter -I FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -t nat -I POSTROUTING -o {x} -j MASQUERADE
sysctl net.ipv4.ip_forward=1");
        run_commands(command.as_str());

        // add device to list if setup worked
        list.push(Device {
            interface: Interface::System,
            name: sys.name().parse().unwrap(),

            iface: Arc::new(Mutex::from(Option::from(sys))),
            radio: Arc::new(Mutex::from(None)),
        });
    }


    // create radio device
    let sdr = RadioStream::new();

    if let Ok(..) = sdr {
        list.push(Device {
            interface: Interface::SDR,
            name: "SDR".to_string(),

            iface: Arc::new(Mutex::new(None)),
            radio: Arc::new(Mutex::from(Option::from(sdr.unwrap()))),
        })
    }

    list
}

/// Network stream of raw communication to and from network devices
pub struct NetworkStream {
    device: Device, // Device that is being used

    mtu: Vec<u8>, // buffer to store data if using network interface
}

/// Network stream of raw communication to and from network devices
impl NetworkStream {
    /// Initialize a NetworkStream object
    pub fn new(device: Device) -> NetworkStream {
        NetworkStream { device, mtu: vec![0; 1500] }
    }

    /// Send data through network device
    pub fn send(&mut self, data: &[u8]) {
        if self.device.interface == Interface::System {

            // Handle sending data if network port is preferred
            self.device.iface.lock().unwrap().as_mut().unwrap().send(data).expect("Error trying to send data through network port");
        } else {

            // Handle if SDR is preferred
            self.device.radio.lock().unwrap().as_mut().unwrap().transmit(data).expect("Error trying to send data through SDR");
        }
    }

    /// Retrieve data from network port
    pub fn receive(&mut self) -> Vec<u8> {
        if self.device.interface == Interface::System {

            // Handle receiving data if network port is preferred
            self.device.iface.lock().unwrap().as_mut().unwrap().recv(self.mtu.as_mut_slice()).expect("Error trying to send data through network port");

            self.mtu.clone()
        } else {

            // Handle if SDR is preferred
            self.device.radio.lock().unwrap().as_mut().unwrap().read().expect("Error trying to send data through SDR")
        }
    }
}