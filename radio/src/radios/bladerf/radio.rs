use std::mem;
use std::ptr::null;
use crate::radios::bladerf::bindings::{bladerf, bladerf_open};
use crate::radios::bladerf::stream::{BladeRFRxStream, BladeRFTxStream};

pub struct BladeRF{
    _dev: *mut bladerf,
    rx_streams: u8,
    tx_streams: u8,
}

impl Default for BladeRF{
    fn default() -> Self {
        unsafe {
            let identifier = null();

            let mut to_return = BladeRF {
                _dev: mem::uninitialized(),
                rx_streams: 0,
                tx_streams: 0,
            };

            bladerf_open(&mut to_return._dev as _, identifier);

            to_return
        }
    }
}

impl BladeRF{
    pub fn create_tx_stream(&mut self)->BladeRFTxStream{
        let hold = BladeRFTxStream::new(self._dev,self.tx_streams);

        self.tx_streams += 1;

        hold
    }

    pub fn create_rx_stream(&mut self)->BladeRFRxStream{
        let hold = BladeRFRxStream::new(self._dev,self.rx_streams);

        self.rx_streams += 1;

        hold
    }

}