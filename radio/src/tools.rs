/// u8 array to binary string
pub fn u8_to_bin(arr: &[u8]) -> String {
    let mut binary_string = String::new();

    for &byte in arr {
        let binary_byte = format!("{:08b}", byte);
        binary_string.push_str(&binary_byte);
    }

    binary_string
}

/// binary string to u8 array
pub fn bin_to_u8(bin: &str) -> Vec<u8> {
    let mut to_return = Vec::new();

    let mut hold = String::from("");

    let mut chars = bin.chars();

    // Split at every 8 digits ( to form 1 byte )
    for x in 0..bin.len() {
        unsafe {
            let next_char = chars.next().unwrap_unchecked();
            hold.push(next_char);

            if x % 8 == 7 {
                let radix = u8::from_str_radix(hold.as_str(), 2).unwrap_unchecked();
                to_return.push(radix);

                hold.clear();
            }
        }
    }

    to_return
}
