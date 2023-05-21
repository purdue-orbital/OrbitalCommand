use std::process::Command;
use std::thread;
use std::time::Duration;
use tun_tap::{Iface, Mode};
use net::layer_3::icmp::{IcmpTypes, ICMPv4};
use net::layer_3::ipv4::Address;

#[test]
pub fn live_test()
{
    let source_ip = Address::from_str("10.107.1.3").unwrap();
    let dest_ip = Address::from_str("127.0.0.1").unwrap();

    let identifier:u16 = 6987;
    let seq_num:u16 = 1;

    let mut packet = ICMPv4::new(IcmpTypes::EchoRequest,64, source_ip, dest_ip, (identifier as u32) << 16 | seq_num as u32, &[0; 46]);

    let sys = Iface::without_packet_info("tun0", Mode::Tun).unwrap();


    let arr = ["addr", "add", "dev", sys.name(), "10.107.1.3/24"];
    Command::new("ip").args(arr.as_slice()).spawn().unwrap().wait().unwrap();

    let arr = ["link", "set", "up", "dev", sys.name()];
    Command::new("ip").args(arr.as_slice()).spawn().unwrap().wait().unwrap();

    let mut encoded = packet.encode(false);

    //encoded.extend_from_slice(&[0, 0, 8, 0]);



    println!("{}",packet.encode(true).len());

    let mut buffer = vec![0; 1504];



    loop {
        thread::sleep(Duration::from_secs(1));

        let size = sys.send(encoded.as_slice()).unwrap();

        println!("{size}");
    }
}

#[test]
pub fn encode_decode() {
    let mut x = ICMPv4::new(
        IcmpTypes::EchoRequest,128,
        Address::from_str("192.168.1.4").unwrap(),
        Address::from_str("192.168.1.4").unwrap(),
        u32::MAX - 1,
        &[]
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
        IcmpTypes::EchoRequest,128,
        Address::from_str("192.168.1.4").unwrap(),
        Address::from_str("192.168.1.4").unwrap(),
        21780,
        &[]
    );

    // this should be true
    assert!(x.verify(),"Failed to verify checksum as true!");

    // make an unchecked update
    x.message_type = 4;

    // this should be false
    assert!(!x.verify(),"Failed to verify checksum as false!");

    // update checksum
    x.update_checksum();

    // this should be true
    assert!(x.verify(),"Failed to verify checksum as true after update!");
}