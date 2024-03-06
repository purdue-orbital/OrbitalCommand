use radio::OrbitalRadio;

fn main() {
    let radio = OrbitalRadio::default();

    loop {
        radio.send(&[1.0]);
        dbg!(radio.fetch());
    }
    
}