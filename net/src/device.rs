use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{spawn};

use tun_tap::{Iface, Mode};

use radio::RadioStream;

use anyhow::{Error, Result};

use crate::interface::Interface;
use crate::interface::Interface::{ETHER, System, WLAN};
use crate::layer_3::ipv4::{Address};
use crate::layer_4::tcp::TCPv4;
use crate::layer_4::udp::UDPv4;
use crate::services::Service;
use crate::tools::run_commands;

fn new_read_thread(iface: Arc<RwLock<Option<Iface>>>) -> Receiver<(usize, [u8; 1500])> {
    // This is for communicating with the read thread
    let (thread_tx, thread_rx) = mpsc::channel();

    spawn(move||{
        let mut mtu = [0; 1500];

        let size = iface.read().unwrap().as_ref().unwrap().recv(mtu.as_mut_slice()).unwrap();

        thread_tx.send((size, mtu))
    });

    thread_rx
}

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

    pub protocols: Vec<Arc<RwLock<Option<Box<dyn Service + Send>>>>>,
    pub ports: Vec<Arc<RwLock<Option<Box<dyn Service + Send>>>>>,

    pub ip_addr: Option<Address>,
    pub dns_addr: Option<Address>,
    //pub gateway: Option<Address>,

    //-----------------------------------
    // Physical Devices
    //-----------------------------------

    pub iface: Arc<RwLock<Option<Iface>>>,
    //pub radio: Arc<RwLock<Option<RadioStream>>>,

    tx: Option<Sender<( Vec<Arc<RwLock<Option<Box<dyn Service + Send>>>>>, Vec<Arc<RwLock<Option<Box<dyn Service + Send>>>>>, Arc<RwLock<Option<Iface>>>)>>
}

impl Device {
    /// Start deice for sending/receiving traffic
    pub fn start(&mut self){
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

        let (tx, rx) = mpsc::channel();
        tx.send((self.ports.clone(), self.protocols.clone(), self.iface.clone())).unwrap();
        self.tx = Some(tx);

        // start listen thread
        spawn(move ||{

            // Get initial values
            let empty_vec = Vec::new();

            let (mut ports, mut proto, mut iface) = rx.recv().unwrap();

            let mut thread_rx = new_read_thread(iface.clone());

            loop {

                // check for updates on protocols
                if let Ok(x) = rx.try_recv(){
                    // ide throws an error here but no error actually occurs
                    (ports, proto, iface) = x;
                }

                // check read channel
                let buff =


                if let Ok(x) = thread_rx.try_recv(){

                    thread_rx = new_read_thread(iface.clone());

                    // return packet
                    x.1[..x.0].to_vec()
                }else{
                    // return nothing by default
                    empty_vec.clone()
                };

                // ensure it is a proper IPv4 packet
                if !buff.is_empty() && buff[0] >> 4 == 4 {

                    // Check protocols (non port numbers)
                    if proto.get(buff[9] as usize).unwrap().read().unwrap().is_some() {

                        // run service
                        if proto[buff[9] as usize].write().unwrap().as_mut().unwrap().run_service(buff.as_slice()){
                            // service asks to be disabled, disable
                            proto[buff[9] as usize].write().unwrap().take();
                        };

                        // Check if UDP
                    } else if buff[9] == 17 {
                        let udp = UDPv4::decode(buff.as_slice()).unwrap();

                        // ensure port is open
                        if let Some(x) = ports[udp.dst_port as usize].write().unwrap().as_mut() {
                            x.run_service(buff.as_slice());
                        }

                        // Check if tcp
                    } else if buff[9] == 6 {
                        let tcp = TCPv4::decode(buff.as_slice()).unwrap();

                        // ensure port is open
                        if let Some(x) = ports[tcp.dst_port as usize].write().unwrap().as_mut()  {
                            x.run_service(buff.as_slice());
                        }
                    }
                }
            }
        }
        );

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
    pub fn set_ip(&mut self, gateway: &str, ip: &str) -> Result<()>{
        self.ip_addr = Some(Address::from_str(ip)?);

        let command = format!("ip addr add dev {} local {gateway} remote {ip}", self.name);

        run_commands(command.as_str());

        Ok(())
    }

    /// Add a new service to listen on a given port number (Event based)
    ///
    /// # Parameter(s):
    /// - 'service' - the service that will be set to listen on a given port number
    /// - 'host_port_num' - port to use
    /// # Error(s):
    /// 1. Returns an error if port is already in use
    pub fn add_listen_service(&mut self, service: Box<dyn Service + Send>, host_port_num:u16) -> Result<()>{
        let hold = if self.ports.get(host_port_num as usize).unwrap().read().unwrap().is_some(){
            Err(Error::msg("Port in use"))
        }else {
            self.ports.insert(host_port_num as usize,Arc::new(RwLock::from(Some(service))));
            Ok(())
        };

        //send update if device is running and had been updated
        if hold.is_ok() && self.tx.is_some(){
            self.tx.as_mut().unwrap().send((self.ports.clone(), self.protocols.clone(), self.iface.clone())).unwrap();
        }

        hold
    }

    /// Stop a service from listening on a given port number
    ///
    /// # Parameter(s):
    /// - 'host_port_num' - port to stop
    pub fn stop_listen_service(&mut self, host_port_num:u16){
        if self.ports.get(host_port_num as usize).is_some(){
            // disable port
            self.ports[host_port_num as usize].write().unwrap().take();
        }

        self.tx.as_mut().unwrap().send((self.ports.clone(), self.protocols.clone(), self.iface.clone())).unwrap();
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
        let hold = if self.protocols.get(protocol_num as usize).unwrap().read().unwrap().is_some(){
            Err(Error::msg("Port in use"))
        }else {
            self.protocols[protocol_num as usize] = Arc::new(RwLock::from(Some(service)));
            Ok(())
        };

        //send update if device is running and had been updated
        if hold.is_ok() && self.tx.is_some(){
            self.tx.as_mut().unwrap().send((self.ports.clone(), self.protocols.clone(), self.iface.clone())).unwrap();
        }

        hold
    }

    /// Stop a service from listening fro a given protocol number
    ///
    /// # Parameter(s):
    /// - 'host_port_num' - port to stop
    pub fn stop_listen_service_without_port(&mut self, protocol_num:u16){
        if self.protocols.get(protocol_num as usize).unwrap().read().unwrap().is_some(){
            // disable listening
            self.protocols[protocol_num as usize].write().unwrap().take();
        }

        self.tx.as_mut().unwrap().send((self.ports.clone(), self.protocols.clone(), self.iface.clone())).unwrap();
    }

    // /// This will run a service once on startup then listen
    // ///
    // /// # Parameter(s):
    // /// - 'service' - the service that will be run once then be set to listen
    // /// - 'host_port_num' - port to use
    // /// - 'data' - initial data that is passed to service
    // /// # Error(s):
    // /// 1. Returns an error if port is already in use
    // pub fn run_and_listen(&mut self, service: Box<dyn Service + Send>, host_port_num:u16 , data: &[u8]) -> Result<()>{
    //     self.add_listen_service(service, host_port_num)?;
    //
    //     self.ports[host_port_num as usize].read().unwrap().as_ref().unwrap().run_service(data);
    //
    //     Ok(())
    // }
}

impl AsRef<Device> for Device {
    fn as_ref(&self) -> &Self {  // Self is Struct<'a>, the type for which we impl AsRef
        self
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

    let empty = vec![Arc::new(RwLock::from(None)); u16::MAX as usize];

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

            protocols: empty.clone(),
            ports: empty.clone(),

            ip_addr: None,
            dns_addr: None,
            iface: Arc::new(RwLock::new(Some(sys))),
            //radio: Arc::new(RwLock::from(None)),

            tx: None,
        });
    }


    // create radio device
    let sdr = RadioStream::new();

    if let Ok(..) = sdr {
        list.push(Device {
            interface: Interface::SDR,
            name: "SDR".to_string(),
            interface_name: "".to_string(),

            protocols: empty.clone(),
            ports: empty.clone(),

            ip_addr: None,
            dns_addr: None,
            iface: Arc::new(RwLock::new(None)),
            //radio: Arc::new(RwLock::from(Option::from(sdr.unwrap()))),

            tx: None,
        })
    }

    list
}