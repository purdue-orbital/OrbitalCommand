use radio::dsp::{convolution, generate_wave, GoertzelAlgorithm, Tools};

#[test]
pub fn convolution_test(){
    let out1 = convolution(&[2.0,1.0,0.0,1.0], &[1.0,2.0,3.0,1.0]);
    let expected1 = vec![2.0,5.0,8.0,6.0,3.0,3.0,1.0];


    let out2 = convolution(&[0.0,1.0,2.0], &[1.0,1.0,1.0,1.0]);
    let expected2 = vec![0.0,1.0,3.0,3.0,3.0,2.0];


    assert_eq!(out1, expected1, "Convolution test 1 failed!");
    assert_eq!(out2, expected2, "Convolution test 2 failed!");
}

#[test]
pub fn resolution_test() {
    // This is a pattern
    let pat = [true,false,false,true,true,true,true,false,false,false];

    // initialize tools
    let tool = Tools::new(10,1024);
    let algo = GoertzelAlgorithm::new(1024_f32,1e6,1e3);

    // Generate wave from preset pattern
    let mut wave = Vec::new();
    for x in pat {
        wave.extend(
            if x {
                generate_wave(1e3,1e6,10,0,1.0,0.0,0.0)
            }else{
                generate_wave(1e5,1e6,10,0,1.0,0.0,0.0)
            }
        )
    }

    // increase the resolution for DFT
    let new_wave = tool.increase_resolution(wave.as_slice());

    // preform dfts and generate pattern mask
    let mut mask = Vec::new();
    for x in (0..new_wave.len()).step_by(1024){
         mask.push(
             algo.run_optimized(&new_wave[x..x+1024]) > 9.0
         )
    }

    // assert patterns match
    assert_eq!(pat, mask.as_slice());
}

