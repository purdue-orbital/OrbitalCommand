use std::sync::{Arc, Mutex};
use std::thread::spawn;

use tun_tap::{Iface, Mode};

use radio::RadioStream;

use anyhow::{Error, Result};

use crate::interface::Interface;
use crate::interface::Interface::{ETHER, System, WLAN};
use crate::layer_3::ipv4::IPV4;
use crate::services::Service;
use crate::tools::run_commands;

/// This is a instance of a device that can be connected to
#[derive(Clone)]
pub struct Device {

    //-----------------------------------
    // Basic Device information
    //-----------------------------------

    pub interface: Interface,
    pub name: String,
    pub interface_name: String,

    //-----------------------------------
    // Internet based information
    //-----------------------------------

    pub protocols: Vec<Arc<Mutex<Option<Box<dyn Service + Send>>>>>,
    pub ports: Vec<Arc<Mutex<Option<Box<dyn Service + Send>>>>>,

    //-----------------------------------
    // Physical Devices
    //-----------------------------------

    pub iface: Arc<Mutex<Option<Iface>>>,
    pub radio: Arc<Mutex<Option<RadioStream>>>,
}

impl Device {
    /// Setup deice for sending/receiving traffic
    pub fn initialize(&mut self){
        // startup instance command
        let startup = format!(
            "ip link set dev {} up", self.name
        );

        // run startup
        run_commands(startup.as_str());

        // Setup port to accept and send traffic
        let command = format!("iptables -t filter -I FORWARD -i {} -o {} -j ACCEPT
iptables -t filter -I FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -t nat -I POSTROUTING -o {} -j MASQUERADE
sysctl net.ipv4.ip_forward=1", self.name, self.interface_name, self.interface_name);
        run_commands(command.as_str());

        let iface = self.iface.clone();
        let protos = self.protocols.clone();

        // start listen thread
        spawn(move ||{
            let mut mtu = [0; 1500];

            loop {
                // get mtu
                let size = iface.lock().unwrap().as_mut().unwrap().recv(mtu.as_mut_slice()).unwrap();

                // decode packet
                let packet = IPV4::decode(mtu.as_mut_slice());

                // ensure it is proper
                if packet.is_ok(){

                    // unwrap
                    let packet = packet.unwrap();

                    // Check protocols (non port numbers)
                    if protos.get(packet.protocol as usize).unwrap().lock().unwrap().is_some(){
                        // run service
                        protos[packet.protocol as usize].lock().unwrap().as_mut().unwrap().run_service(&mtu[..size]);
                    }
                }
            }
        });
    }

    /// Set gateway and the IP address of this device
    /// # Important:
    /// Due to limitations of TUN/TAP, a virtual local area network (VLAN) is created on this system
    /// and all virtual network interface cards (VNICs) will will be "plugged into" the vlan. So all
    /// devices should have the same gateway but be given different IPs.
    ///
    /// # Parameter(s):
    /// - 'gateway' - this the gateway to all VNICs on this system (EX: 192.168.69.0/24)
    /// - 'ip' - IP address of this device on the VLAN (EX: 192.168.69.1)
    pub fn set_ip(&self, gateway: &str, ip: &str) {
        let command = format!("ip addr add dev {} local {gateway} remote {ip}", self.name);

        run_commands(command.as_str());
    }

    /// Add a new service to listen on a given port number (Event based)
    ///
    /// # Parameter(s):
    /// - 'service' - the service that will be set to listen on a given port number
    /// - 'host_port_num' - port to use
    /// # Error(s):
    /// 1. Returns an error if port is already in use
    pub fn add_listen_service(&mut self, service: Box<dyn Service + Send>, host_port_num:u16) -> Result<()>{
        if self.ports.get(host_port_num as usize).unwrap().lock().unwrap().is_some(){
            Err(Error::msg("Port in use"))
        }else {
            self.ports.insert(host_port_num as usize,Arc::new(Mutex::new(Option::from(service))));
            Ok(())
        }
    }

    /// Stop a service from listening on a given port number
    ///
    /// # Parameter(s):
    /// - 'host_port_num' - port to stop
    pub fn stop_listen_service(&mut self, host_port_num:u16){
        if self.ports.get(host_port_num as usize).unwrap().lock().unwrap().is_some(){
            // disable port
            self.ports[host_port_num as usize] = Arc::new(Mutex::new(None));
        }
    }

    /// Stop a service from listening fro a given protocol number
    ///
    /// # Parameter(s):
    /// - 'host_port_num' - port to stop
    pub fn stop_listen_service_without_port(&mut self, protocol_num:u16){
        if self.protocols.get(protocol_num as usize).unwrap().lock().unwrap().is_some(){
            // disable listening
            self.protocols[protocol_num as usize] = Arc::new(Mutex::new(None));
        }
    }

    /// Add a new service to listen on a given protocol (Event based) This is intended for services
    /// that don't use port numbers like ICMP
    ///
    /// # Parameter(s):
    /// - 'service' - the service that will be set to listen on a given port number
    /// - 'host_port_num' - port to use
    /// # Error(s):
    /// 1. Returns an error if port is already in use
    pub fn add_listen_service_without_port(&mut self, service: Box<dyn Service + Send>, protocol_num:u16) -> Result<()>{
        if self.protocols.get(protocol_num as usize).unwrap().lock().unwrap().is_some(){
            Err(Error::msg("Port in use"))
        }else {
            self.protocols[protocol_num as usize] = Arc::new(Mutex::new(Option::from(service)));
            Ok(())
        }
    }

    /// This will run a service once on startup then listen
    ///
    /// # Parameter(s):
    /// - 'service' - the service that will be run once then be set to listen
    /// - 'host_port_num' - port to use
    /// - 'data' - initial data that is passed to service
    /// # Error(s):
    /// 1. Returns an error if port is already in use
    pub fn run_and_listen(&mut self, service: Box<dyn Service + Send>, host_port_num:u16 , data: &[u8]) -> Result<()>{
        self.add_listen_service(service, host_port_num)?;

        self.ports[host_port_num as usize].lock().unwrap().as_mut().unwrap().run_service(data);

        Ok(())
    }
}

/// Return all network ports, such as an ethernet port, on this device
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

        let interface = match x.get(0..1).unwrap() {
            "e" => ETHER,
            "w" => WLAN,
            _ => System,
        };

        // add device to list if setup worked
        list.push(Device {
            interface,
            name: sys.name().parse().unwrap(),
            interface_name: x,

            protocols: vec![Arc::new(Mutex::new(None)); u16::MAX as usize],
            ports: vec![Arc::new(Mutex::new(None)); u16::MAX as usize],

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
            interface_name: "".to_string(),

            protocols: vec![Arc::new(Mutex::new(None)); u16::MAX as usize],
            ports: vec![Arc::new(Mutex::new(None)); u16::MAX as usize],

            iface: Arc::new(Mutex::new(None)),
            radio: Arc::new(Mutex::from(Option::from(sdr.unwrap()))),
        })
    }

    list
}