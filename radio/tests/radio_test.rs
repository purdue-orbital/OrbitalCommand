use radio::radios::bladerf::{Radio, Stream};

#[test]
fn test_radio(){
    let mut radio:Radio<f32> = Radio::new().unwrap();
    let mut rx_stream = radio.create_rx_stream(0);

    rx_stream.set_sample_rate(20_000_000).unwrap();
    rx_stream.set_lo_frequency(915_000_000).unwrap();
    rx_stream.set_gain_auto().unwrap();
}