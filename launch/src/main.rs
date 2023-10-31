#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use ds323x::Ds323x;
use mpu9250_i2c::Mpu9250;
use rppal::{i2c::I2c, hal::Delay};

fn main() {
    let mut rtc = Ds323x::new_ds3231(I2c::new().unwrap());
    let mut mpu = Mpu9250::new(I2c::new().unwrap(), Delay, Default::default()).unwrap();
    
    let packet: [u8; 28] = CfgPrtUartBuilder {
        portid: UartPortId::Uart1,
        reserved0: 0,
        tx_ready: 0,
        mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
        baud_rate: 9600,
        in_proto_mask: InProtoMask::all(),
        out_proto_mask: OutProtoMask::UBLOX,
        flags: 0,
        reserved5: 0,
     }.into_packet_bytes();
    
    println!("Hello, world!");
}
