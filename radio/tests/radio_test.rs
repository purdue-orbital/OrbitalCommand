use num_complex::Complex;
use radio::radios::bladerf::radio::BladeRF;

#[test]
fn test_radio(){
    let mut radio = BladeRF::default();

    let r = radio.create_rx_stream();
    let t = radio.create_tx_stream();

    r.set_gain_auto().unwrap();
    r.set_frequency(916_000_000).unwrap();
    r.set_sample_rate(20_000_000).unwrap();


    //t.set_gain_auto().unwrap();
    t.set_frequency(916_000_000).unwrap();
    t.set_sample_rate(20_000_000).unwrap();

    loop{
        t.tx(vec![Complex::new(1.0,0.0);10000].as_slice());
    }
    let out = r.rx(100);

    dbg!(out);
}