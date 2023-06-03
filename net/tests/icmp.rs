use std::thread;
use std::time::Duration;

use net::{list_devices, NetworkStream};
use net::layer_3::icmp::{IcmpTypes, ICMPv4};
use net::layer_3::ipv4::Address;

#[test]
pub fn live_test()
{
    let source_ip = Address::from_str("192.168.69.1").unwrap();
    let dest_ip = Address::from_str("8.8.8.8").unwrap();

    let identifier: u16 = 642;
    let mut seq_num: u16 = 1;

    let mut packet = ICMPv4::new(IcmpTypes::EchoRequest, 64, source_ip, dest_ip, (identifier as u32) << 16 | seq_num as u32, &[0; 56]);

    let dev = list_devices().get(0).unwrap().clone();

    dev.set_ip("192.168.69.0/24", "192.168.69.1");

    // Create network stream
    let mut stream = NetworkStream::new(dev);

    // send packets
    loop {
        thread::sleep(Duration::from_secs(1));

        let  encoded = packet.encode(false);

        stream.send(encoded.as_slice());

        seq_num += 1;
        packet.rest_of_header = (identifier as u32) << 16 | seq_num as u32;

        packet.update_checksum();
    }
}

#[test]
pub fn encode_decode() {
    let mut x = ICMPv4::new(
        IcmpTypes::EchoRequest, 128,
        Address::from_str("192.168.1.4").unwrap(),
        Address::from_str("192.168.1.4").unwrap(),
        u32::MAX - 1,
        &[],
    );

    // encode
    let encode = x.encode(false);

    // decode
    let decode = ICMPv4::decode(encode.as_slice());

    // make sure different parts are encoded and decoded properly
    assert_eq!(decode.message_type, x.message_type);
}


#[test]
pub fn checksum() {
    let mut x = ICMPv4::new(
        IcmpTypes::EchoRequest, 128,
        Address::from_str("192.168.1.4").unwrap(),
        Address::from_str("192.168.1.4").unwrap(),
        21780,
        &[],
    );

    // this should be true
    assert!(x.verify(), "Failed to verify checksum as true!");

    // make an unchecked update
    x.message_type = 4;

    // this should be false
    assert!(!x.verify(), "Failed to verify checksum as false!");

    // update checksum
    x.update_checksum();

    // this should be true
    assert!(x.verify(), "Failed to verify checksum as true after update!");
}