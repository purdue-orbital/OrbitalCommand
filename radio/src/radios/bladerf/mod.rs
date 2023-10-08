mod bindings;
mod stream;

use std::marker::PhantomData;
use std::ops::{Add, Div, Mul, Sub};
use anyhow::{Error, Result};
use std::os::raw::{c_char};
use num::cast::AsPrimitive;
use num::FromPrimitive;
use num_complex::Complex;
use crate::radios::bladerf::bindings::*;
use crate::radios::bladerf::stream::*;


pub struct RxStream<T:GenericStream + Copy + Mul<Output = T>  + AsPrimitive<i16> + FromPrimitive + Add<Output = T> + Sub<Output = T> + Div<Output = T>>{ stream: Stream<T>, }

impl<T:GenericStream + Copy + Mul<Output = T>  + AsPrimitive<i16> + FromPrimitive + Add<Output = T> + Sub<Output = T> + Div<Output = T>> RxStream<T> {
    pub fn new(device: *mut bladerf,channel:i32) -> RxStream<T>{
        RxStream{
            stream: Stream::new(device, Direction::RX, channel)
        }
    }

    pub fn set_lo_frequency(&mut self, frequency: u64) -> Result<()> {
        self.stream.set_lo_frequency(frequency)
    }

    pub fn set_gain(&mut self, gain:i32) -> Result<()>{
        self.stream.set_gain(gain)
    }

    pub fn set_gain_auto(&mut self) -> Result<()> {
        self.stream.set_gain_auto()
    }

    pub fn set_sample_rate(&mut self, sample_rate:u32) -> Result<()> {
        self.stream.set_sample_rate(sample_rate)
    }

    pub fn rx(&mut self, arr: &mut [Complex<T>], timeout_ms:u32){
        self.stream.rx(arr,timeout_ms);
    }
}

pub struct TxStream<T:GenericStream + Copy + Mul<Output = T>  + AsPrimitive<i16> + FromPrimitive + Add<Output = T> + Sub<Output = T> + Div<Output = T>>{ stream: Stream<T>, }
impl<T:GenericStream + Copy + Mul<Output = T>  + AsPrimitive<i16> + FromPrimitive + Add<Output = T> + Sub<Output = T> + Div<Output = T>> TxStream<T> {
    pub fn new(device: *mut bladerf,channel:i32) -> TxStream<T>{
        TxStream{
            stream: Stream::new(device, Direction::TX, channel)
        }
    }

    pub fn set_lo_frequency(&mut self, frequency: u64) -> Result<()> {
        self.stream.set_lo_frequency(frequency)
    }

    pub fn set_gain(&mut self, gain:i32) -> Result<()>{
        self.stream.set_gain(gain)
    }

    pub fn set_gain_auto(&mut self) -> Result<()> {
        self.stream.set_gain_auto()
    }

    pub fn set_sample_rate(&mut self, sample_rate:u32) -> Result<()> {
        self.stream.set_sample_rate(sample_rate)
    }

    pub fn tx(&mut self, arr: &mut [Complex<T>], timeout_ms:u32){
        self.stream.tx(arr,timeout_ms)
    }
}


pub struct Radio<T:GenericStream + Copy + Mul<Output = T>  + AsPrimitive<i16> + FromPrimitive + Add<Output = T> + Sub<Output = T> + Div<Output = T>> {
    device: *mut bladerf,

    marker: PhantomData<T>,
}

impl<T: GenericStream + Copy + Mul<Output = T> + AsPrimitive<i16> + FromPrimitive + Add<Output = T> + Sub<Output = T> + Div<Output = T>> Radio<T>{
    pub fn new() -> Result<Radio<T>> {
        // create the bladeRF struct
        let device_identifier = c_char::from(0); // just an empty string (yes I know it looks like a char, but C is dumb)

        unsafe {
            let mut to_return = Radio {
                device: &mut bladerf {
                    _unused: [],
                } as *mut _,
                marker: PhantomData::default()
            };

            if bladerf_open(&mut to_return.device, &device_identifier) == 0 {
                Ok(to_return)
            } else {
                Err(Error::msg("Unable to connect to a bladeRF device"))
            }
        }
    }

    pub fn create_tx_stream(&self,channel:i32) -> TxStream<T> {
        TxStream::new(self.device,channel)
    }

    pub fn create_rx_stream(&self,channel:i32) -> RxStream<T> {
        RxStream::new(self.device,channel)
    }

    pub fn close(&self) {
        unsafe {bladerf_close(self.device);}
    }
}