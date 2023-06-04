use ux::u13;

use net::layer_3::ipv4::{Address, AssuredForwarding, DifferentiatedServices, ECN, IPPrecedence, IPV4};

#[test]
pub fn ipv4_address() {
    // test from string
    let test1 = Address::from_str("192.168.1.4").unwrap();

    // test encode/decode
    let test2 = Address::decode(test1.encode());

    // test verbatim creation
    let test3 = Address::new(192, 168, 1, 4);

    // Run tests
    assert_eq!(test1.to_string(), "192.168.1.4".to_string(), "Failed to create from string");
    assert_eq!(test2.to_string(), "192.168.1.4".to_string(), "Failed to encode and decode");
    assert_eq!(test3.to_string(), "192.168.1.4".to_string(), "Failed to create from verbatim");
}


#[test]
pub fn encode_decode() {
    let x = IPV4::new(
        [255, 255, 255, 255, 255, 255, 255, 255, 255, 255].as_slice(),
        [255, 255, 255, 255, 255, 255, 255, 255, 255, 255].as_slice(),

        // Picking this option, specifically CS7, for production use is a really bad idea to do but
        // in terms of the encoding process, this option would be the hardest to encode
        &DifferentiatedServices::new(IPPrecedence::CS7, AssuredForwarding::AFx2),
        ECN::new(true, true),
        u16::MAX,
        u13::MAX,
        u8::MAX,
        u8::MAX,
        u32::MAX,
        u32::MAX,
    );

    // encode
    let encode = x.encode(false);

    // decode
    let decode = IPV4::decode(encode.as_slice());

    // make sure different parts are encoded and decoded properly
    assert_eq!(decode.source_ip_address, x.source_ip_address);
    assert_eq!(decode.differentiated_services_code_point, x.differentiated_services_code_point);
    assert_eq!(decode.explicit_congestion_notification, x.explicit_congestion_notification);
    assert_eq!(decode.option, x.option);
    assert_eq!(decode.get_data(), x.get_data());
}


#[test]
pub fn checksum() {
    let mut x = IPV4::new(
        [].as_slice(),
        [].as_slice(),
        &DifferentiatedServices::new(IPPrecedence::CS2, AssuredForwarding::AFx0),
        ECN::new(false, false),
        1,
        u13::new(0),
        64,
        0,
        50,
        u32::MAX - 1,
    );

    // this should be true
    assert!(x.verify(), "Failed to verify checksum as true!");

    // make an unchecked update
    x.protocol = 1;

    // this should be false
    assert!(!x.verify(), "Failed to verify checksum as false!");

    // update checksum
    x.update_checksum();

    // this should be true
    assert!(x.verify(), "Failed to verify checksum as true after update!");
}
