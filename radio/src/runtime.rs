use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn};
use std::time::Duration;

use flume::Receiver;
use rustdsp::Modulators;

use crate::radio_settings::RadioSetting;
use crate::radios::bladerf::radio::BladeRF;
use crate::radios::bladerf::stream::BladeRFTxStream;

pub struct Runtime {
    radio_setting: RadioSetting,
    tx_stream: BladeRFTxStream,
    rx_channel: Receiver<u8>,
    modulator: Modulators,
}

impl Runtime {
    pub fn new(settings: RadioSetting) -> anyhow::Result<Runtime> {

        // Connect to radio
        let radio = Arc::new(RwLock::new(BladeRF::default()));

        let tx_stream = radio.write().unwrap().create_tx_stream();

        // set tx settings
        tx_stream.set_frequency(settings.lo_frequency as u64)?;
        tx_stream.set_gain(50)?;
        tx_stream.set_sample_rate(settings.sample_rate as u64)?;
        tx_stream.dc_calibrate();

        // create rx channel
        let (tx, rx) = flume::unbounded();

        let samples_per_a_symbol = (settings.sample_rate / settings.baud_rate) as usize;

        // Create RX loop
        spawn(move || {
            let d = rustdsp::Demodulators::new(samples_per_a_symbol, settings.baud_rate);

            let rx_stream = radio.write().unwrap().create_rx_stream();

            // set rx settings
            rx_stream.set_frequency(settings.lo_frequency as u64).unwrap();
            rx_stream.set_gain_auto().unwrap();
            rx_stream.set_sample_rate(settings.sample_rate as u64).unwrap();
            rx_stream.enable_pll();
            rx_stream.dc_calibrate();

            loop {
                // // sample
                // let mtu = rx_stream.rx(samples_per_a_symbol);
                //
                // // demod
                // let bit = d.bpsk(mtu)[0];
                //
                // bin <<= 1;
                // bin |= bit;
                // bin_counter += 1;
                //
                // if bin_counter == 8{
                //
                // }

                // let mut bin_counter: u8 = 0;
                let mut bin: u8 = 0;

                for _ in 0..8 {
                    let mtu = rx_stream.rx(samples_per_a_symbol);
                    let bit = d.bpsk(mtu)[0];

                    bin <<= 1;
                    bin |= bit;
                }

                tx.send(bin).unwrap();
            }
        });


        Ok(Runtime {
            radio_setting: settings,
            tx_stream,
            rx_channel: rx,
            modulator: Modulators::new(samples_per_a_symbol, settings.sample_rate),
        })
    }

    pub fn tx(&self, bin: &[u8]) {
        // modulate
        let modulated = self.modulator.bpsk(bin);

        // send
        self.tx_stream.tx(modulated.as_slice());
    }

    pub fn rx(&self) -> Vec<u8> {
        let mut buffer = vec![];

        for x in self.rx_channel.iter(){
            buffer.push(x);

            if buffer.len() > 1_000{
                return buffer;
            }
        }

        buffer
    }
}