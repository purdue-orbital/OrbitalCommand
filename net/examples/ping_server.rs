use std::thread;
use std::time::Duration;
use net::device::list_devices;
use net::services::Ping;

fn main(){
    // Test settings
    let gateway = "192.168.69.0/24";
    let ip = "192.168.69.1";

    // Create/find a device
    let mut device = list_devices()[0].clone();

    // create ping service
    let ping = Ping::new(&mut device).unwrap();

    // add service to device
    device.add_listen_service_without_port(Box::from(ping), 1).unwrap();

    // start device
    device.initialize();

    // set IP and gateway
    device.set_ip(gateway,ip);

    // notify
    println!("Ping service started! Try pinging {ip}!");

    // start endless loop
    loop {
        thread::sleep(Duration::from_secs(10));
    }
}