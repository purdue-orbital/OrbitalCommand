use std::str::Chars;

use num_complex::Complex;

/// Subtract values with the value to their left. This will remove 1 element from the array size
///
/// # Arguments
/// * `arr` - Array Of Values To Preform Operations
///
pub fn subtract_left_adjacent(arr: Vec<f32>) -> Vec<f32>
{
    // Left shift values
    let mut out = Vec::new();

    for x in 1..arr.len()
    {
        out.push(arr.clone().get(x - 1).unwrap() - arr.clone().get(x).unwrap())
    }

    out
}


/// Find the moving average of an array of numbers
///
/// # Arguments
/// * `arr` - Array Of Values To Preform Operations
/// * `size` - Number of values to average together to form average
///
pub fn moving_average(arr: Vec<f32>, size: usize) -> Vec<f32>
{
    // return vector
    let mut out = Vec::new();

    // place holder array to calculate average by
    let mut to_sum = Vec::new();

    for x in arr {
        // add value to array
        to_sum.push(x);

        // if array is the size to preform average on, preform moving average
        if to_sum.len() == size {

            // calculate average and add to array
            out.push(to_sum.iter().sum::<f32>() / size as f32);

            // remove first index
            to_sum.pop();
        }
    }

    out
}

pub fn average_complex(arr: Vec<Complex<f32>>) -> f32 {
    let mut sum = 0.0;
    for x in arr.clone() {
        sum += x.norm_sqr().sqrt()
    }

    sum / arr.len() as f32
}

/// Normalize values of an array of numbers
///
/// # Arguments
/// * `arr` - Array Of Values To Preform Operations
/// * `size` - Number of values to average together to form average
///
pub fn normalize(arr: Vec<f32>) -> Vec<f32>
{
    // Get max
    let max = *arr.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Get min
    let min = *arr.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Normalize values and return
    arr.iter().map(|&x| (x - min) / (max - min)).collect()
}

/// Although the format! macro does this for us, we sometimes want to dynamically set the fixed
/// length of the formatted binary
///
/// # Arguments
/// * `num` - Number to
/// * `len` - Fixed length of binary
#[inline]
pub fn i32_to_char_bin(num: i32, len: usize) -> Vec<char> {
    let mut to_return = Vec::with_capacity(len);

    for i in (0..(len) as i32).rev() {
        let k = num >> i;

        if (k & 1) == 1 {
            to_return.push('1')
        } else {
            to_return.push('0')
        }
    }

    to_return
}


/// Although the format! macro does this for us, we sometimes want to dynamically set the fixed
/// length of the formatted binary
///
/// # Arguments
/// * `num` - Number to
/// * `len` - Fixed length of binary
#[inline]
pub fn bin_char_arr_to_usize_unchecked(bin: Chars) -> usize {
    let mut total = 0;
    let mut multiplier = 1;

    for x in bin.rev() {
        if x == '1' {
            total += multiplier;
        }

        multiplier *= 2;
    }

    total
}


/// u8 array to binary string
pub fn u8_to_bin(arr: &[u8]) -> String {
    let mut binary_string = String::new();

    for &byte in arr {
        let binary_byte = format!("{:08b}", byte);
        binary_string.push_str(&binary_byte);
    }

    binary_string
}

/// binary string to u8 array
pub fn bin_to_u8(bin: &str) -> Vec<u8> {
    let mut to_return = Vec::new();

    let mut hold = String::from("");

    let mut chars = bin.chars();

    // Split at every 8 digits ( to form 1 byte )
    for x in 0..bin.len() {

        if let Some(next_char) = chars.next(){
            hold.push(next_char)
        }

        if x % 8 == 7 {

            if let Ok(radix) = u8::from_str_radix(hold.as_str(), 2){
                to_return.push(radix);
            }

            hold.clear();
        }
    }

    to_return
}

pub fn flip_bin(bin: &mut String) -> String{
    let mut to_return = String::with_capacity(bin.len());

    for x in bin.chars(){
        if x == '1'{
            to_return += "0";
        }else {
            to_return += "1";
        }
    }

    to_return
}

