use num_complex::Complex;
use rustfft::{FftDirection, FftPlanner};
use radio::dsp::filters::fir;
use radio::dsp::filters::fir::shapes::WindowShapes;
use radio::dsp::tools::generate_wave::generate_wave;

// #[test]
// fn test_rectangle() {
//     // set wave settings
//     let fft_size = 1024;
//     let sample_rate = 1e6;
//     let frequency = sample_rate / 2.0;
//
//     // create wave settings
//     let mut fft:FftPlanner<f32> = FftPlanner::new();
//     let forward = fft.plan_fft_forward(fft_size);
//     let reverse = fft.plan_fft_inverse(fft_size);
//
//     // make window
//     let window = fir::Windowing::new(WindowShapes::Rectangle, fft_size, 0);
//
//     // generate wave
//     let mut wave = generate_wave(frequency, sample_rate, fft_size as i32, 0, 1.0, 0.0, 0.0);
//
//     let og = wave.clone();
//
//     // preform fft
//     forward.process(wave.as_mut_slice());
//     window.run(wave.as_mut_slice());
//     reverse.process(wave.as_mut_slice());
//
//     // normalize
//     let wave:Vec<_> = wave.iter().map(|&x| x.norm() / fft_size as f32).collect();
//
//     assert_eq!(og,wave);
// }