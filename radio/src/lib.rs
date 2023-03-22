pub mod stream;
pub mod radio;
pub mod dsp;
pub mod graphy;

#[derive(PartialEq)]
pub struct FrequencyRange
{
    pub center_frequency: f64,
    pub lpf_bandwidth: f64,
}

// This function can be used for help setting explicit ranges of frequencies you want the radio to listen on
pub fn frequency_range(start_frequency: f64, stop_frequency: f64) -> FrequencyRange
{
    FrequencyRange { center_frequency: (start_frequency + stop_frequency) / 2.0, lpf_bandwidth: (stop_frequency - start_frequency) }
}