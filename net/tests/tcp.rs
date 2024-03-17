use net::layer_3::ipv4::Address;
use net::layer_4::tcp::{TCPv4, TcpFlags};

#[test]
pub fn encode_decode() {
    let mut x = TCPv4::new(
        Address::from_str("192.168.1.1").unwrap(),
        22,
        Address::from_str("192.168.1.4").unwrap(),
        22,
        [4, 4, 4, 4, 4, 4, 4].as_slice(),
        [3].as_slice(),
        [TcpFlags::Ack].as_slice(),
        0,
        1,
        1024,
        0,
    );

    // encode
    let encode = x.encode(false);

    // decode
    let decode = TCPv4::decode(encode.as_slice()).unwrap();

    // make sure different parts are encoded and decoded properly
    assert_eq!(decode.dst_port, x.dst_port);
    assert_eq!(decode.src_port, x.src_port);
    assert_eq!(decode.data, x.data);
    assert_eq!(decode.ipv4.source_ip_address, x.ipv4.source_ip_address);
}

#[test]
pub fn checksum() {
    let mut x = TCPv4::new(
        Address::from_str("192.168.1.1").unwrap(),
        22,
        Address::from_str("192.168.1.4").unwrap(),
        22,
        [4, 4, 4, 4, 4, 4, 4].as_slice(),
        [3].as_slice(),
        [TcpFlags::Ack].as_slice(),
        0,
        1,
        1024,
        0,
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
    assert!(
        x.verify(),
        "Failed to verify checksum as true after update!"
    );
}
