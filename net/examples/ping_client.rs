use std::thread::sleep;
use std::time::Duration;
use net::device::list_devices;
use net::layer_3::ipv4::Address;
use net::services::{Ping};

fn main(){
    // Test settings
    let gateway = "192.168.69.0/24";
    let ip = "192.168.69.1";

    // Create/find a device
    let mut device = list_devices()[0].clone();

    // set IP and gateway
    device.set_ip(gateway,ip).unwrap();

    // start device
    device.start();

    // Make ping instance
    let mut ping = Ping::new(&mut device);

    // ping google 10 times
    for _ in 0..10{
        // send ping
        let stat = ping.ping(Address::from_str("8.8.8.8").unwrap()).unwrap();

        // print stats if completed
        if stat.completed{
            println!("{} bytes from {}: icmp_seq={} hops={} time={} ms", stat.length, stat.resp_addr, stat.seq_num, stat.hops, stat.duration.as_micros() as f64 / 1000.0);
        }

        // wait 900ms (ping already waits 100ms)
        sleep(Duration::from_millis(900));
    }
}