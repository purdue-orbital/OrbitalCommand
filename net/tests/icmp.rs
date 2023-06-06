use net::layer_3::icmp::{IcmpTypes, ICMPv4};
use net::layer_3::ipv4::Address;

#[test]
pub fn encode_decode() {
    let mut x = ICMPv4::new(
        IcmpTypes::EchoRequest, 128,
        Address::from_str("192.168.1.4").unwrap(),
        Address::from_str("192.168.1.4").unwrap(),
        u32::MAX - 1,
        &[5],
    );

    // encode
    let encode = x.encode(false);

    // decode
    let decode = ICMPv4::decode(encode.as_slice()).unwrap();

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