use std::num::Wrapping;
use std::str::Chars;

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
pub fn u32_to_char_bin(num: u32, len: usize) -> Vec<char> {
    let mut to_return = Vec::with_capacity(len);

    for i in (0..(len) as u32).rev(){
        let k = num >> i;

        if (k & 1) == 1 {
            to_return.push('1')
        }else{
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

/// Calculate sum of an array of u16s while adding carries to the beginning
pub fn sum_with_carries(to_sum:&[u16]) -> u16 {
    // keep track of the number of carries
    let mut carries = 0;

    // sum all values
    let mut sum:u16 = 0;

    for &x in to_sum{

        // check for a carry
        let to_check = sum.checked_add(x);

        // add to carry counter if a carry happened
        if to_check.is_none(){
            carries += 1;
        }

        sum = (Wrapping(sum) + Wrapping(x)).0;
    }

    // check for a carry
    let to_check = sum.checked_add(carries);

    // add to carry counter if a carry happened
    if to_check.is_none(){
        carries += 1;
    }

    (Wrapping(sum) + Wrapping(carries)).0
}


/// Convert an array of u8s to an array of u16s
pub fn u8_arr_to_u16_arr(arr:&[u8]) -> Vec<u16>{
    let mut to_return = Vec::with_capacity(arr.len() / 2);

    for x in (1..arr.len()).step_by(2){
        to_return.push(((arr[x-1] as u16) << 8) | (arr[x] as u16))
    }

    to_return
}