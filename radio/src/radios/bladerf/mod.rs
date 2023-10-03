mod bindings;

use std::{mem, ptr};
use std::ffi::{c_int, c_void};
use std::marker::PhantomData;
use std::ops::{Add, Div, Mul, Sub};
use anyhow::{Error, Result};
use std::os::raw::{c_char, c_uint};
use num::cast::AsPrimitive;
use num_complex::Complex;
use crate::radios::bladerf::bindings::*;

#[derive(PartialEq)]
pub enum Direction{
    RX,
    TX
}

#[derive(PartialEq)]
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
}

impl GenericStream for i16 {
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11_META,
            desired_format: Formats::I16,
        }
    }

}

impl GenericStream for u16 {
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11_META,
            desired_format: Formats::U16,
        }
    }

}

impl GenericStream for i8{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC8_Q7_META,
            desired_format: Formats::I8,
        }
    }

}

impl GenericStream for u8{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC8_Q7_META,
            desired_format: Formats::U8,
        }
    }

}


impl GenericStream for f32{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11_META,
            desired_format: Formats::F32,
        }
    }
}

impl GenericStream for f64{
    fn return_format_type() -> StreamFormat {
        StreamFormat{
            format: bladerf_format_BLADERF_FORMAT_SC16_Q11_META,
            desired_format: Formats::F64,
        }
    }
}


pub struct Stream<T: GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>> {
    pub direction: Direction,
    pub channel: i32,

    pub blade_channel: bladerf_channel,

    pub meta: bladerf_metadata,

    pub device: *mut bladerf,

    marker: PhantomData<T>
}


impl<T:GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>> Stream<T> {
    pub fn new(device: *mut bladerf,direction: Direction, channel:i32) -> Stream<T>{
        // get blade's name for channels
        let blade_channel =
            if direction == Direction::RX {
                if channel == 1{
                    bladerf_channel_layout_BLADERF_RX_X1
                }else {
                    bladerf_channel_layout_BLADERF_RX_X2
                }
            }else if channel == 1{
                bladerf_channel_layout_BLADERF_TX_X1
            }else {
                bladerf_channel_layout_BLADERF_TX_X2
            };

        unsafe {
            bladerf_sync_config(device,blade_channel,T::return_format_type().format,32 as c_uint,4096 as c_uint, 32 as c_uint,1000 as c_uint);

            // start stream (our code is designed to be synchronous so this will be always true)
            bladerf_enable_module(device,blade_channel as bladerf_channel,true);
        }

        Stream{
            direction,
            channel,
            blade_channel: blade_channel as bladerf_channel,
            meta: bladerf_metadata {
                timestamp: 0,
                flags: BLADERF_META_FLAG_TX_BURST_END,
                status: 0,
                actual_count: 0,
                reserved: [0;32],
            },
            device,
            marker: PhantomData::default(),
        }
    }

    /// This function takes data we have received from a sample and sets them, in place, with the
    /// intended output
    pub fn convert_from(&self, input:*mut c_void, output:&mut [Complex<T>]){

        let arr:&[i16] = unsafe {
             std::slice::from_raw_parts(input as *const i16,output.len() * 2)
        };

        for (index, x) in output.iter_mut().enumerate() {
            x.re = T::from(arr[index * 2]);
            x.im = T::from(arr[(index * 2) + 1]);
        }

        if T::return_format_type().desired_format == Formats::F64 || T::return_format_type().desired_format == Formats::F32 {
            output.iter_mut().for_each(|x| {x.re = x.re / T::from(2048); x.im = x.im / T::from(2048)});
        } else if T::return_format_type().desired_format == Formats::U16{
            output.iter_mut().for_each(|x| {x.re = x.re + T::from(2048); x.im = x.im + T::from(2048)});
        }else if T::return_format_type().desired_format == Formats::U8{
            output.iter_mut().for_each(|x| {x.re = x.re + T::from(256); x.im = x.im + T::from(256)});
        }

    }


    /// Preps data for transmission
    pub fn convert_to(&self, input:*mut c_void, output:&mut [Complex<T>]){

        let mut arr:&mut [i16] = unsafe {
            std::slice::from_raw_parts_mut(input as *mut i16,2 * output.len())
        };

        for (index, x) in output.iter().enumerate() {
            arr[index * 2] = (x.re).as_();
            arr[(index * 2) + 1] = (x.im).as_();
        }

        if T::return_format_type().desired_format == Formats::F64 || T::return_format_type().desired_format == Formats::F32 {
            arr.iter_mut().for_each(|x| *x *= 2048_i16);
        } else if T::return_format_type().desired_format == Formats::U16{
            arr.iter_mut().for_each(|x| *x -= 2048_i16);
        }else if T::return_format_type().desired_format == Formats::U8{
            arr.iter_mut().for_each(|x| *x -= 256_i16);
        }
    }

    pub fn rx(&mut self, arr:&mut [Complex<T>], timeout_ms: u32){

        let samples = if T::return_format_type().format == bladerf_format_BLADERF_FORMAT_SC16_Q11{
            vec![0i16; arr.len() * 2].as_mut_ptr() as *mut _
        }else {
            vec![0i8; arr.len() * 2].as_mut_ptr() as *mut _
        };

        unsafe {
            bladerf_sync_rx(
                self.device,
                samples,
                (arr.len() * 2) as _,
                &mut self.meta as *mut _,
                timeout_ms as c_uint
            );
        }

        self.convert_from(samples, arr);

    }
    pub fn tx(&mut self, arr:&mut [Complex<T>], timeout_ms: u32){

        let samples = if T::return_format_type().format == bladerf_format_BLADERF_FORMAT_SC16_Q11{
            vec![0i16; arr.len() * 2].as_mut_ptr() as *mut _
        }else {
            vec![0i8; arr.len() * 2].as_mut_ptr() as *mut _
        };

        self.convert_to(samples,arr);


        unsafe {
            bladerf_sync_tx(
                self.device,
                samples,
                arr.len() as c_uint,
                &mut self.meta as *mut _,
                timeout_ms
            );
        }

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

pub struct RxStream<T:GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>>{ stream: Stream<T>, }

impl<T:GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>> RxStream<T> {
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

pub struct TxStream<T:GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>>{ stream: Stream<T>, }
impl<T:GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>> TxStream<T> {
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


pub struct Radio<T:GenericStream + From<i16> + Copy + Mul<Output = T>  + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>> {
    device: *mut bladerf,

    marker: PhantomData<T>,
}

impl<T: GenericStream + From<i16> + Copy + Mul<Output = T> + AsPrimitive<i16> + Add<Output = T> + Sub<Output = T> + Div<Output = T>> Radio<T>{
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
        dbg!("Test4.1");

        unsafe {bladerf_close(self.device);}

        dbg!("Test4.2");
    }

}

