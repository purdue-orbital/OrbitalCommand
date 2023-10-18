use std::ffi::c_uint;
use std::ptr::null_mut;

use anyhow::Error;
use num_complex::Complex;

use crate::common::{f32_complex_to_i12_iq, i12_iq_to_f32_complex};
use crate::radios::bladerf::bindings::{bladerf, bladerf_channel, bladerf_channel_layout, bladerf_channel_layout_BLADERF_RX_X1, bladerf_channel_layout_BLADERF_RX_X2, bladerf_channel_layout_BLADERF_TX_X1, bladerf_channel_layout_BLADERF_TX_X2, bladerf_enable_module, bladerf_format_BLADERF_FORMAT_SC16_Q11, bladerf_gain, bladerf_gain_mode_BLADERF_GAIN_HYBRID_AGC, bladerf_set_frequency, bladerf_set_gain, bladerf_set_gain_mode, bladerf_set_sample_rate, bladerf_sync_config, bladerf_sync_rx, bladerf_sync_tx};

struct BladeRFStream {
    _dev: *mut bladerf,
    channel: bladerf_channel,
}

impl BladeRFStream {
    pub fn new(device: *mut bladerf, channel: bladerf_channel) -> BladeRFStream {
        unsafe {
            bladerf_enable_module(device, channel, true);

            bladerf_sync_config(device, channel as bladerf_channel_layout, bladerf_format_BLADERF_FORMAT_SC16_Q11, 32, 1024, 16, 1000);

            BladeRFStream {
                _dev: device,
                channel,
            }
        }
    }

    pub fn set_frequency(&self, frequency: u64) -> anyhow::Result<()> {
        unsafe {
            if bladerf_set_frequency(self._dev, self.channel, frequency as _) == 0 {
                Ok(())
            } else {
                Err(Error::msg("Error while setting the frequency"))
            }
        }
    }

    pub fn set_sample_rate(&self, sample_rate: u64) -> anyhow::Result<()> {
        unsafe {
            if bladerf_set_sample_rate(self._dev, self.channel, sample_rate as _, null_mut()) == 0 {
                Ok(())
            } else {
                Err(Error::msg("Error while setting the sample rate"))
            }
        }
    }

    pub fn set_gain_auto(&self) -> anyhow::Result<()> {
        unsafe {
            if bladerf_set_gain_mode(self._dev, self.channel, bladerf_gain_mode_BLADERF_GAIN_HYBRID_AGC) == 0 {
                Ok(())
            } else {
                Err(Error::msg("Error while enabling auto gain mode"))
            }
        }
    }


    pub fn set_gain(&self, gain: u8) -> anyhow::Result<()> {
        unsafe {
            if bladerf_set_gain(self._dev, self.channel, gain as bladerf_gain) == 0 {
                Ok(())
            } else {
                Err(Error::msg("Error while enabling auto gain mode"))
            }
        }
    }

    pub fn rx(&self, nsamples: usize) -> Vec<Complex<f32>> {
        let mut temp_mtu = vec![0_i16; nsamples * 2];

        unsafe {
            bladerf_sync_rx(self._dev, temp_mtu.as_mut_ptr() as _, nsamples as c_uint, null_mut(), 1000);
        }

        i12_iq_to_f32_complex(temp_mtu.as_slice())
    }


    pub fn tx(&self, mtu: &[Complex<f32>]) {
        let mut temp_mtu = f32_complex_to_i12_iq(mtu);

        unsafe {
            bladerf_sync_tx(self._dev, temp_mtu.as_mut_ptr() as _, mtu.len() as c_uint, null_mut(), 1000);
        }
    }
}

pub struct BladeRFRxStream {
    stream: BladeRFStream,
}

impl BladeRFRxStream {
    pub fn new(device: *mut bladerf, channel: u8) -> BladeRFRxStream {
        let blade_channel = if channel == 0 {
            bladerf_channel_layout_BLADERF_RX_X1
        } else {
            bladerf_channel_layout_BLADERF_RX_X2
        };

        let stream = BladeRFStream::new(device, blade_channel as bladerf_channel);

        BladeRFRxStream {
            stream
        }
    }

    pub fn set_frequency(&self, frequency: u64) -> anyhow::Result<()> {
        self.stream.set_frequency(frequency)
    }

    pub fn set_sample_rate(&self, sample_rate: u64) -> anyhow::Result<()> {
        self.stream.set_sample_rate(sample_rate)
    }

    pub fn set_gain_auto(&self) -> anyhow::Result<()> {
        self.stream.set_gain_auto()
    }

    pub fn rx(&self, nsamples: usize) -> Vec<Complex<f32>> {
        self.stream.rx(nsamples)
    }
}

pub struct BladeRFTxStream {
    stream: BladeRFStream,
}

impl BladeRFTxStream {
    pub fn new(device: *mut bladerf, channel: u8) -> BladeRFTxStream {
        let blade_channel = if channel == 0 {
            bladerf_channel_layout_BLADERF_TX_X1
        } else {
            bladerf_channel_layout_BLADERF_TX_X2
        };

        let stream = BladeRFStream::new(device, blade_channel as bladerf_channel);

        BladeRFTxStream {
            stream
        }
    }

    pub fn set_frequency(&self, frequency: u64) -> anyhow::Result<()> {
        self.stream.set_frequency(frequency)
    }

    pub fn set_sample_rate(&self, sample_rate: u64) -> anyhow::Result<()> {
        self.stream.set_sample_rate(sample_rate)
    }

    pub fn set_gain(&self, gain: u8) -> anyhow::Result<()> {
        self.stream.set_gain(gain)
    }

    pub fn tx(&self, mtu: &[Complex<f32>]) {
        self.stream.tx(mtu)
    }
}
