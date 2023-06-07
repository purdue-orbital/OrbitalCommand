use std::thread;
use std::time::Duration;
use net::device::list_devices;
use net::services::{Ping};

fn main(){
    // Test settings
    let gateway = "192.168.69.0/24";
    let ip = "192.168.69.1";

    // Create/find a device
    let mut device = list_devices()[0].clone();

    // set IP and gateway
    device.set_ip(gateway,ip);

    // start device
    device.start();

    // Make ping instance
    let mut ping = Ping::new(&mut device);

    // Enable ping
    ping.enable();

    // notify
    println!("Ping service started! Try pinging {ip}!");

    // start endless loop
    loop {
        thread::sleep(Duration::from_secs(10));
    }
}