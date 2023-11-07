extern crate crc;

use crc::crc32;

pub fn calculate_checksum(data: &[u8]) -> [u8; 4] {
    let checksum = crc32::checksum_ieee(data);
    let bytes: [u8; 4] = checksum.to_be_bytes();
    bytes
}
/*pub fn calculate_checksum(data: &[u8]) -> u8 {
    // Implement your checksum algorithm here
    // This is a simple example using a basic checksum calculation
    let checksum = data.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte));
    !checksum
}*/