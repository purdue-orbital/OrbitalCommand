use std::thread;
use std::time::Duration;

use net::{list_devices, NetworkStream};
use net::layer_3::ipv4::Address;
use net::layer_4::udp::UDPv4;

// #[test]
// pub fn live_test()
// {
//     let source_ip = Address::from_str("192.168.69.1").unwrap();
//     let dest_ip = Address::from_str("192.168.69.2").unwrap();
//
//     let mut packet = UDPv4::new(source_ip,22,dest_ip,22, [5].as_slice());
//
//     let dev = list_devices().get(0).unwrap().clone();
//
//     dev.set_ip("192.168.69.0/24", "192.168.69.1");
//
//     // Create network stream
//     let mut stream = NetworkStream::new(dev);
//
//     // send packets
//     loop {
//         thread::sleep(Duration::from_secs(1));
//
//         let  encoded = packet.encode(false);
//
//         stream.send(encoded.as_slice());
//     }
// }

#[test]
pub fn encode_decode() {
    let mut x = UDPv4::new(
        Address::from_str("192.168.1.1").unwrap(),
        22,
        Address::from_str("192.168.1.4").unwrap(),
        22,
        [4, 4, 4, 4, 4, 4, 4].as_slice(),
    );

    // encode
    let encode = x.encode(false);

    // decode
    let decode = UDPv4::decode(encode.as_slice());

    // make sure different parts are encoded and decoded properly
    assert_eq!(decode.dst_port, x.dst_port);
    assert_eq!(decode.src_port, x.src_port);
    assert_eq!(decode.data, x.data);
    assert_eq!(decode.ipv4.source_ip_address, x.ipv4.source_ip_address);
}


#[test]
pub fn checksum() {
    let mut x = UDPv4::new(
        Address::from_str("192.168.1.1").unwrap(),
        22,
        Address::from_str("192.168.1.4").unwrap(),
        22,
        [4, 4, 4, 4, 4, 4, 4].as_slice(),
    );

    // this should be true
    assert!(x.verify(), "Failed to verify checksum as true!");

    // make an unchecked update
    x.src_port = 1;
    x.ipv4.protocol = 4;

    // this should be false
    assert!(!x.verify(), "Failed to verify checksum as false!");

    // update checksum
    x.update_checksum();

    // this should be true
    assert!(x.verify(), "Failed to verify checksum as true after update!");
}
