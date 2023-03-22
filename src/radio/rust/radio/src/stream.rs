use std::collections::HashMap;

use num_complex::Complex;
use soapysdr::{Args, Direction};
use anyhow::Result;

use crate::radio::Radio;

pub struct Stream
{
    // Stream Properties
    radio: Radio,
    method: Direction,
    channel: i8,

}

pub struct TxStream {
    stream: soapysdr::TxStream<Complex<f32>>,
}

pub struct RxStream {
    stream: soapysdr::RxStream<Complex<f32>>,
    buffer: Vec<Complex<f32>>
}

impl RxStream {
    pub fn rx(&mut self) -> Vec<Complex<f32>>
    {
        let mut buff: &mut [Complex<f32>; 1024 as usize] = &mut [Complex::<f32>::new(0.0, 0.0); 1024 as usize];

        self.stream.read(&[buff], 100000).expect("Collect stream");

        self.buffer.extend(buff.iter().copied());

        self.buffer.clone()
    }

    pub fn clear_buffer(&mut self)
    {
        self.buffer = Vec::new();
    }
}

impl TxStream {
    pub fn tx(&mut self, arr: Vec<Complex<f32>>) -> Result<()>
    {
        self.stream.write(&[arr.as_slice()], None, true, 10000)?;

        Ok(())
    }
}

impl Stream {
    pub fn new_tx(radio: Radio, channel: i8, center_frequency: f64, lpf_bandwidth: f64, sample_rate: f64) -> Result<TxStream> {
        let stream = Self::new_stream(radio, Direction::Tx, channel, center_frequency, lpf_bandwidth, sample_rate)?;

        let mut tx = stream
            .radio
            .get_radio()
            .tx_stream::<Complex<f32>>(
                &[channel.try_into().unwrap()])?;

        tx.activate(None)?;

        Ok(TxStream {
            stream: tx,
        })
    }

    pub fn new_rx(radio: Radio, channel: i8, center_frequency: f64, lpf_bandwidth: f64, sample_rate: f64) -> Result<RxStream> {
        let stream = Self::new_stream(radio, Direction::Tx, channel, center_frequency, lpf_bandwidth, sample_rate)?;

        let mut rx = stream
            .radio
            .get_radio()
            .rx_stream::<Complex<f32>>(
                &[channel.try_into().unwrap()])?;

        rx.activate(Some(1000))?;

        Ok(RxStream {
            stream: rx,
            buffer: Vec::new(),
        })
    }

    fn new_stream(radio: Radio, method: Direction, channel: i8, center_frequency: f64, lpf_bandwidth: f64, sample_rate: f64) -> Result<Stream>
    {
        // Make stream data
        let mut new_stream = Stream { radio, method, channel };

        // Stream arguements
        let mut args = Args::new();

        // Set general stream data
        new_stream.radio.get_radio().set_bandwidth(new_stream.method, new_stream.channel.try_into().unwrap(), lpf_bandwidth).expect("Setting Bandwidth");
        new_stream.radio.get_radio().set_frequency(new_stream.method, new_stream.channel.try_into().unwrap(), center_frequency, args).expect("Setting Frequency");
        new_stream.radio.get_radio().set_gain(new_stream.method, new_stream.channel.try_into().unwrap(), 80.0).expect("Setting Gain");
        new_stream.radio.get_radio().set_sample_rate(new_stream.method, new_stream.channel.try_into().unwrap(), sample_rate).expect("Setting Sample Rate");


        // Initialize stream for either TX or RX operations
        // if new_stream.method == Direction::Tx {
        //
        //     // Set TX Stream
        //     new_stream.tx = Some(
        //         new_stream
        //             .radio
        //             .get_radio()
        //             .tx_stream::<Complex<f32>>(
        //                 &[channel.try_into().unwrap()])
        //             .expect("Get TX Stream"));
        //
        //     new_stream.tx.as_mut().unwrap().activate(None).unwrap();
        // } else {
        //     // Set RX Stream
        //     new_stream.rx = Some(
        //         new_stream
        //             .radio
        //             .get_radio()
        //             .rx_stream::<Complex<f32>>(
        //                 &[channel.try_into().unwrap()])
        //             .expect("Get TX Stream"));
        //
        //     new_stream.rx.as_mut().unwrap().activate(Some(1000)).unwrap();
        // }

        Ok(new_stream)
    }
}