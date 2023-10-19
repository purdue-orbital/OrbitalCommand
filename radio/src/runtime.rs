use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn};
use std::time::Duration;

use flume::{Receiver, Sender};
use bytes::Bytes;
use rustdsp::Modulators;

use crate::radio_settings::RadioSetting;
use crate::radios::bladerf::radio::BladeRF;
use crate::radios::bladerf::stream::BladeRFTxStream;

pub struct Runtime {
    radio_setting: RadioSetting,
    tx_stream: BladeRFTxStream,
    /// the flume channel we use to send data onto the next step of the pipeline
    tx_bits: Sender<u8>,
    /// the flume channel we use to get bytes to be sent
    rx_bytes: Receiver<Bytes>,
    modulator: Modulators,
}

impl Runtime {
    pub fn new(settings: RadioSetting, bytes_channel: Receiver<Bytes>) -> anyhow::Result<(Runtime, Receiver<u8>)> {

        // Connect to radio
        let radio = Arc::new(RwLock::new(BladeRF::default()));

        let tx_stream = radio.write().unwrap().create_tx_stream();

        // set tx settings
        tx_stream.set_frequency(settings.lo_frequency as u64)?;
        tx_stream.set_gain(50)?;
        tx_stream.set_sample_rate(settings.sample_rate as u64)?;

        // create rx channel
        let (tx_bits, rx) = flume::unbounded();

        let samples_per_a_symbol = (settings.sample_rate / settings.baud_rate) as usize;

        // Create RX loop

        Ok((
            Runtime {
                radio_setting: settings,
                tx_stream,
                tx_bits,
                rx_bytes: bytes_channel,
                modulator: Modulators::new(samples_per_a_symbol, settings.sample_rate),
            },
            rx
        ))
    }

    // pub fn tx(&self, bin: &[u8]) {
    //     // modulate
    //     let modulated = self.modulator.bpsk(bin);

    //     // send
    //     self.tx_stream.tx(modulated.as_slice());
    // }

    // pub fn rx(&self) -> Vec<u8> {
    //     let mut out = self.rx_channel.recv().unwrap();

    //     while out.is_empty() {
    //         out = self.rx_channel.recv().unwrap();

    //         sleep(Duration::from_millis(1));
    //     }

    //     out
    // }

    pub fn run(self) {
        // recieving thread
        spawn(move || {
            let d = rustdsp::Demodulators::new(samples_per_a_symbol, settings.baud_rate);

            let rx_stream = radio.write().unwrap().create_rx_stream();

            // set rx settings
            rx_stream.set_frequency(settings.lo_frequency as u64).unwrap();
            rx_stream.set_gain_auto().unwrap();
            rx_stream.set_sample_rate(settings.sample_rate as u64).unwrap();

            loop {
                let mut bin: u8 = 0;

                for _ in 0..8 {
                    let mtu = rx_stream.rx(samples_per_a_symbol);
                    let bit = d.bpsk(mtu)[0];

                    bin <<= 1;
                    bin |= bit;
                }

                self.tx_channel.send(bin).unwrap();
            }
        });

        spawn(move || {
            while let Some(bin) = self.rx_bytes.recv() {
                (&self).modulator.bpsk(bin.to_vec());
                (&self).
            }
        });
    }
}