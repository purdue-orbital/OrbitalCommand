use radio::dsp::Modulators;
use radio::dsp::tools::goertzel_algorithm::GoertzelAlgorithm;

static SAMPLE_RATE: f32 = 1e5;
static BAUD_RATE: f32 = 1e4;

#[test]
pub fn goertzel() {
    let samples_per_symbol = (SAMPLE_RATE / BAUD_RATE) as usize;
    let mods = Modulators::new(samples_per_symbol, SAMPLE_RATE);

    let ones = mods.fsk(&[255]);
    let zeros = mods.fsk(&[0]);

    let algo = GoertzelAlgorithm::new(samples_per_symbol as f32, SAMPLE_RATE, SAMPLE_RATE * 0.5);
    let algo_clone = algo.clone();

    assert_eq!(algo.run_optimized(&ones[..samples_per_symbol]).round(), algo.run(&ones[..samples_per_symbol]).round());
    assert_eq!(algo.run_optimized(&zeros[..samples_per_symbol]).round(), algo.run(&zeros[..samples_per_symbol]).round());

    assert_eq!(algo.run(&ones[..samples_per_symbol]), algo_clone.run(&ones[..samples_per_symbol]));
    assert_eq!(algo.run(&zeros[..samples_per_symbol]), algo_clone.run(&zeros[..samples_per_symbol]));
}