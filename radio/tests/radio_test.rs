use radio::radios::bladerf::radio::BladeRF;

#[test]
fn test_radio() {
    let baud_rate = 1_000_000;
    let sample_rate = 20_000_000;

    let samples_per_a_symbol = sample_rate / baud_rate;

    let mut radio = BladeRF::default();
    let m = rustdsp::Modulators::new(samples_per_a_symbol, sample_rate as f32);
    let signal = m.bpsk(vec![255, 255, 255, 255, 0, 255].as_slice());


    let r = radio.create_rx_stream();
    let t = radio.create_tx_stream();

    r.set_gain_auto().unwrap();
    r.set_frequency(916_000_000).unwrap();
    r.set_sample_rate(sample_rate as u64).unwrap();

    t.set_gain(50).unwrap();
    t.set_frequency(916_000_000).unwrap();
    t.set_sample_rate(sample_rate as u64).unwrap();

    t.tx(signal.as_slice());

    let out = r.rx(100);
    dbg!(out);
}