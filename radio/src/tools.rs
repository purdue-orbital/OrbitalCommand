use std::collections::VecDeque;

/// Calculate Sum Of A Array
///
/// # Arguments
/// * `arr` - Array Of Values To Calculate sum
/// * `modifier` - A Number To Multiply Values By Before Being Averaged (Default: 1)
///
pub fn sum(arr:Vec<f32>, modifier: Option<f32>) -> f32
{
    let mut total = 0.0;

    for x in arr
    {
        total += x * modifier.unwrap_or(1.0);
    }

    total
}


/// Make an array of averages over time
///
/// # Arguments
/// * `arr` - Array Of Values To Average
/// * `num` - Number Of Values Averaged Together
/// * `modifier` - A Number To Multiply Values By Before Being Averaged (Default: 1)
///
pub fn average_array(arr : Vec<f32>, num : usize, modifier: Option<f32>) -> Vec<f32>
{
    // Make a buffer to store values to average
    let mut buffer = VecDeque::from( arr.split_at(num-1).0.to_vec());

    // Store averaged values in this array
    let mut out:Vec<f32> =  Vec::new();
    out.push(sum(Vec::from(buffer.clone()), modifier) / (num as f32));

    // Average and then append to the output array
    for x in num..arr.len()
    {
        // Remove first value in array
        buffer = buffer.clone();
        buffer.pop_front();

        // Append next value
        buffer.push_back(*arr.get(x).unwrap());

        // Append new average to array
        out.push(sum(Vec::from(buffer.clone()), modifier) / num as f32);
    }

    // Return array of averaged values
    out
}

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