use num_complex::Complex;
use radio::radios::bladerf::Radio;

#[test]
fn test_radio(){

    let sample_rate = 20_000_000;
    let baud_rate = 1_000_000;

    let m = rustdsp::Modulators::new((sample_rate / baud_rate) as usize, sample_rate as f32);
    let mut signal = m.bpsk(vec![255,0,255].as_mut_slice());

    let mut radio: Radio<f32> = Radio::new().unwrap();
    let mut rx_stream = radio.create_rx_stream(0);
    let mut tx_stream = radio.create_tx_stream(0);

    let mut hold = vec![Complex::new(0.0,0.0); 1024];

    rx_stream.set_gain_auto().unwrap();
    rx_stream.set_sample_rate(sample_rate).unwrap();
    rx_stream.set_lo_frequency(916_000_000).unwrap();

    tx_stream.set_gain(70).unwrap();
    tx_stream.set_sample_rate(sample_rate).unwrap();
    tx_stream.set_lo_frequency(916_000_000).unwrap();

    tx_stream.tx(&mut signal, 1000);
    rx_stream.rx(&mut hold, 1000);

    dbg!(hold);

    radio.close();
}