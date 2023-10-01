mod bindings;

use std::{mem, ptr};
use std::marker::PhantomData;
use anyhow::{Error, Result};
use std::os::raw::c_char;
use num_complex::Complex;
use crate::radios::bladerf::bindings::*;

#[derive(PartialEq)]
pub enum Direction{
    RX,
    TX
}

pub enum Formats{
    U16,
    I16,
    U8,
    I8,
    F32,
    F64
}

pub struct StreamFormat{
    pub format: bladerf_format,
    pub desired_format: Formats
}

pub trait GenericStream{
    fn return_format_type() -> StreamFormat;

    fn _blank() -> bool;
}

impl GenericStream for i16 {
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11,
            desired_format: Formats::I16,
        }
    }

    fn _blank() -> bool {
        false
    }
}

impl GenericStream for u16 {
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11,
            desired_format: Formats::U16,
        }
    }

    fn _blank() -> bool {
        false
    }
}

impl GenericStream for i8{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC8_Q7,
            desired_format: Formats::I8,
        }
    }

    fn _blank() -> bool {
        false
    }
}

impl GenericStream for u8{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC8_Q7,
            desired_format: Formats::U8,
        }
    }

    fn _blank() -> bool {
        false
    }
}


impl GenericStream for f32{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11,
            desired_format: Formats::F32,
        }
    }

    fn _blank() -> bool {
        false
    }
}

impl GenericStream for f64{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11,
            desired_format: Formats::F64,
        }
    }

    fn _blank() -> bool {
        false
    }
}

pub struct Stream<T: GenericStream> {
    pub direction: Direction,
    pub channel: i32,

    pub blade_channel: bladerf_channel,

    pub meta: bladerf_metadata,

    pub device: *mut bladerf,

    marker: PhantomData<T>
}


impl<T:GenericStream> Stream<T> {
    pub fn new(device: *mut bladerf,direction: Direction, channel:i32) -> Stream<T>{
        // get blade's name for channels
        let blade_channel =
            if direction == Direction::RX {
                if channel == 1{
                    bladerf_channel_layout_BLADERF_RX_X1 as bladerf_channel
                }else {
                    bladerf_channel_layout_BLADERF_RX_X2 as bladerf_channel
                }
            }else if channel == 1{
                bladerf_channel_layout_BLADERF_TX_X1 as bladerf_channel
            }else {
                bladerf_channel_layout_BLADERF_TX_X2 as bladerf_channel
            };

        unsafe {
            // start stream (our code is designed to be synchronous so this will be always true)
            bladerf_enable_module(device,blade_channel,true);
        }

        Stream{
            direction,
            channel,
            blade_channel,
            meta: bladerf_metadata {
                timestamp: 0,
                flags: 0,
                status: 0,
                actual_count: 0,
                reserved: [0;32],
            },
            device,
            marker: PhantomData::default(),
        }
    }

    pub fn rx(&self, arr:&mut [Complex<T>]){

    }
    pub fn tx(&self, arr:&mut [Complex<T>]){

    }

    pub fn set_lo_frequency(&mut self, frequency: u64) -> Result<()> {
        unsafe {
            if bladerf_set_frequency(self.device, self.blade_channel, frequency) == 0 {
                Ok(())
            } else {
                Err(Error::msg("Error setting frequency"))
            }
        }
    }

    pub fn set_gain(&mut self, gain:i32) -> Result<()>{
        unsafe {
            self.set_gain_mode(bladerf_gain_mode_BLADERF_GAIN_MGC)?;

            if bladerf_set_gain(self.device,self.blade_channel,gain) == 0{
                Ok(())
            }else{
                Err(Error::msg("An error occurred setting the gain"))
            }
        }
    }

    pub fn set_gain_auto(&mut self) -> Result<()> {
        self.set_gain_mode(bladerf_gain_mode_BLADERF_GAIN_DEFAULT)
    }

    pub fn set_sample_rate(&mut self, sample_rate:u32) -> Result<()> {
        unsafe {
            if bladerf_set_sample_rate(self.device,self.blade_channel,sample_rate, ptr::null_mut()) == 0 {
                Ok(())
            }else {
                Err(Error::msg("Error occurred setting sample rate"))
            }
        }

    }

    pub fn set_gain_mode(&mut self, mode: bladerf_gain_mode) -> Result<()>{
        unsafe {
            if bladerf_set_gain_mode(self.device,self.blade_channel,mode) == 0 {
                Ok(())
            }else{
                Err(Error::msg("Error occurred setting the gain mode"))
            }
        }
    }
}

pub struct RxStream<T:GenericStream>{ stream: Stream<T>, }

impl<T:GenericStream> RxStream<T> {
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

    pub fn rx(&mut self, arr: &mut [Complex<T>]){

    }
}

pub struct TxStream<T:GenericStream>{ stream: Stream<T>, }
impl<T:GenericStream> TxStream<T> {
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

    pub fn rx(&mut self, arr: &mut [Complex<T>]){

    }
}


pub struct Radio<T:GenericStream> {
    device: *mut bladerf,

    marker: PhantomData<T>,
}

impl<T:GenericStream> Radio<T>{
    pub fn new() -> Result<Radio<T>> {
        // create the bladeRF struct
        let device_identifier = c_char::from(0); // just an empty string (yes I know it looks like a char, but C is dumb)

        unsafe {
            let mut to_return = Radio { device: mem::uninitialized(), marker: PhantomData::default() };

            if bladerf_open(&mut to_return.device, &device_identifier) == 0 {
                Ok(to_return)
            } else {
                Err(Error::msg("Unable to connect to a bladeRF device"))
            }
        }
    }

    pub fn create_tx_stream(&self,channel:i32) -> TxStream<T>{
        TxStream::new(self.device,channel)
    }

    pub fn create_rx_stream(&self,channel:i32) -> RxStream<T>{
        RxStream::new(self.device,channel)
    }

}

