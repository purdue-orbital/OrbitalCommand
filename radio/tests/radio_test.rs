use num_complex::Complex;
use radio::radios::bladerf::Radio;

#[test]
fn test_radio(){
    let mut radio:Radio<f32> = Radio::new().unwrap();
    let mut rx_stream = radio.create_rx_stream(0);

    let mut hold = vec![Complex::new(0.0,0.0);1024];

    rx_stream.set_sample_rate(20_000_000).unwrap();
    rx_stream.set_gain_auto().unwrap();
    rx_stream.set_lo_frequency(915_000_000).unwrap();

    rx_stream.rx(hold.as_mut_slice(),100);

    dbg!(hold);
}