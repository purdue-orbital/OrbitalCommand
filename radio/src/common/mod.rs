use num_complex::Complex;


/// This will convert i12 IQ values in to a complex representation
pub fn i12_iq_to_f32_complex(arr:&[i16]) -> Vec<Complex<f32>>{
    assert_eq!(arr.len() % 2,0);

    let mut to_return = vec![Complex::new(0.0, 0.0); arr.len()/2];

    for (index, x) in to_return.iter_mut().enumerate(){
        x.re = (arr[2*index] as f32) / 2048.0 ;
        x.im = (arr[(2*index) + 1] as f32) / 2048.0;
    }

    to_return
}

/// This will convert a complex representation to i12 IQ values
pub fn f32_complex_to_i12_iq(arr: &[Complex<f32>]) -> Vec<i16>{
    let mut to_return = vec![0; arr.len()*2];

    for (index, x) in arr.iter().enumerate(){
        to_return[index * 2] = (x.re * 2048.0) as i16;
        to_return[(index * 2) + 1] = (x.im * 2048.0) as i16;
    }

    to_return
}
