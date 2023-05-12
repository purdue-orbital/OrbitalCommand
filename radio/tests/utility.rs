#[test]
fn frame_test() {

    // Test bytes
    let test_arr1 = [4, 252, 112, 128];
    let test_arr2 = [32, 22, 69, 22];

    // Make a frame
    let frame_1 = radio::Frame::new(test_arr1.clone().as_mut_slice());
    let frame_2 = radio::Frame::new(test_arr2.clone().as_mut_slice());

    // Turn the frame into a string
    let mut for_transmission1 = frame_1.assemble();
    let mut for_transmission2 = frame_2.assemble();

    // Add some noise
    for_transmission1 = format!("0000000000110110100000000000000000001100000{for_transmission1}0000000001110000001100101000000010101010{for_transmission2}11010110101010000010110101010");

    // Reassemble
    let mut frame_3 = radio::Frame::from(for_transmission1.as_str());

    // Ensure frames match
    assert_eq!(frame_2.assemble(), frame_3.pop().unwrap().assemble());
    assert_eq!(frame_1.assemble(), frame_3.pop().unwrap().assemble());
}