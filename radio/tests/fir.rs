use once_cell::sync::Lazy;
use num_complex::Complex;
use radio::dsp::filters::fir;
use radio::dsp::filters::fir::shapes::WindowShapes;
use radio::dsp::tools::generate_wave::generate_wave;


// set wave settings
static FFT_SIZE:usize = 1024;
static SAMPLE_RATE:f32 = 1e6;
static FREQUENCY:f32 = SAMPLE_RATE / 2.0;

static SIGNAL:Lazy<Vec<Complex<f32>>> = Lazy::new(|| {
    generate_wave(FREQUENCY, SAMPLE_RATE, FFT_SIZE as i32, 0, 1.0, 0.0, 0.0)
});

fn vector_equal(arr1: Vec<Complex<f32>>,arr2: Vec<Complex<f32>>) -> bool{

    if arr1.len() != arr2.len() {return false}

    for (index, x) in arr1.iter().enumerate(){
        if (arr1[index].norm() - arr2[index].norm()).abs() > 0.001{
            return false
        }
    }

    true
}

#[test]
fn test_rectangle() {
    // make window
    let mut window = fir::Windowing::new(WindowShapes::Rectangle, FFT_SIZE, 0);
    let mut wave = SIGNAL.clone();

    window.run(wave.as_mut_slice());

    assert!(vector_equal(SIGNAL.clone(),wave));
}

#[test]
fn test_triangle() {
    // make window
    let mut window = fir::Windowing::new(WindowShapes::Triangle, FFT_SIZE, 0);
    let mut wave = SIGNAL.clone();

    window.run(wave.as_mut_slice());

    assert!(vector_equal(SIGNAL.clone(),wave));
}

#[test]
fn test_welch() {
    // make window
    let mut window = fir::Windowing::new(WindowShapes::Welch, FFT_SIZE, 0);
    let mut wave = SIGNAL.clone();

    window.run(wave.as_mut_slice());

    assert!(vector_equal(SIGNAL.clone(),wave));
}