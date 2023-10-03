// split a vector and reverse the two halves, combine them, then return that vector
pub fn split_reverse(signal: &mut[Complex<f32>]) -> Vec<Complex<f32>>
{
    let (first_half, second_half) = signal.split_at_mut(signal.len() / 2);

    first_half.reverse();
    second_half.reverse();

    let mut combined = first_half.to_vec();

    combined.append(&mut second_half.to_vec());

    combined
}