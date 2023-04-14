use std::collections::VecDeque;

/// Subtract values with the value to their left. This will remove 1 element from the array size
///
/// # Arguments
/// * `arr` - Array Of Values To Preform Operations
///
pub fn subtract_left_adjacent(arr:Vec<f32>) -> Vec<f32>
{
    // Left shift values
    let mut out = Vec::new();

    for x in 1..arr.len()
    {
        out.push(arr.clone().get(x-1).unwrap() - arr.clone().get(x).unwrap())
    }

    out
}


/// Find the moving average of an array of numbers
///
/// # Arguments
/// * `arr` - Array Of Values To Preform Operations
/// * `size` - Number of values to average together to form average
///
pub fn moving_average(arr:Vec<f32>, size:usize) -> Vec<f32>
{
    // return vector
    let mut out = Vec::new();

    // place holder array to calculate average by
    let mut to_sum = Vec::new();

    for x in arr{
        // add value to array
        to_sum.push(x);

        // if array is the size to preform average on, preform moving average
        if to_sum.len() == size{

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
pub fn normalize(arr:Vec<f32>) -> Vec<f32>
{
    // Get max
    let max = *arr.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Get min
    let min = *arr.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Normalize values and return
    arr.iter().map(|&x| (x - min) / (max - min)).collect()
}