use num_complex::Complex;
use radio::radios::bladerf::Radio;

#[test]
fn test_radio(){

    let sample_rate = 20_000_000;
    let baud_rate = 100_000;

    let m = rustdsp::Modulators::new((sample_rate / baud_rate) as usize, sample_rate as f32);
    let mut signal = m.bpsk(&[255,255,255,255,255,255,255,255,255,255,255,255,255,255,255]);

    let mut radio: Radio<f32> = Radio::new().unwrap();
    let mut rx_stream = radio.create_rx_stream(0);
    let mut tx_stream = radio.create_tx_stream(0);

    let mut hold = vec![Complex::new(0.0,0.0); 1024];

    dbg!("Test1");

    rx_stream.set_gain_auto().unwrap();
    rx_stream.set_sample_rate(20_000_000).unwrap();
    rx_stream.set_lo_frequency(915_000_000).unwrap();

    dbg!("Test2");

    tx_stream.set_gain(70).unwrap();
    tx_stream.set_sample_rate(20_000_000).unwrap();
    tx_stream.set_lo_frequency(915_000_000).unwrap();

    dbg!("Test3");

    tx_stream.tx(&mut signal, 100);
    rx_stream.rx(&mut hold, 100);

    for each in hold.iter().rev() {
        dbg!(each);
    }

    dbg!("Test4");

    radio.close();

    dbg!("Test5");

    dbg!("Test6");

}