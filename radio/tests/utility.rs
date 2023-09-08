
/// u8 array to binary string
fn u8_to_bin(arr: &[u8]) -> String {
    let mut name_in_binary = String::from("");

    for character in arr {
        name_in_binary += &format!("{:08b}", *character);
    }

    name_in_binary
}

#[test]
fn u8_to_bin_test(){
    let bin = [3_u8,5,1,2];
    let expected = "00000011000001010000000100000010".to_string();

    let to_test = u8_to_bin(bin.as_slice());

    assert_eq!(to_test,expected,"u8 to bin check.\n\tGot: {}\n\tExpected: {}", to_test, expected);
}

#[test]
fn frame_test() {

    // Test bytes
    let test_arr1 = [4, 252, 112, 128];

    // Make a frame
    let frame_1 = radio::frame::Frame::new(test_arr1.clone().as_mut_slice());

    // Turn the frame into a string
    let for_transmission1 = u8_to_bin(&frame_1.assemble().as_slice()[10..]);

    // Reassemble
    let mut frame_3 = radio::frame::Frame::from(vec!(for_transmission1));

    // Ensure frames match
    assert_eq!(frame_1.assemble(), frame_3.pop().unwrap().assemble());
}