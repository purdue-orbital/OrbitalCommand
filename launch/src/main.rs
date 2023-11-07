#![deny(clippy::unwrap_used)]

use std::{process::Command, str::FromStr};

use chrono::{NaiveDateTime, Utc, DateTime};
use ds323x::{Ds323x, DateTimeAccess};
use mpu9250_i2c::Mpu9250;
use rppal::{i2c::I2c, hal::Delay};
use serialport::SerialPort;
use ublox::{CfgPrtUartBuilder, UartPortId, UartMode, DataBits, Parity, StopBits, InProtoMask, OutProtoMask};
use clap::{Parser as ClapParser, Subcommand};
use log::{warn, error};
use ublox::*;

const DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Clock {
        #[command(subcommand)]
        command: ClockCommands,
    }
}

#[derive(Debug, Subcommand)]
enum ClockCommands {
    /// Set the current RTC time with an ISO8601 string and exit
    Set {
        time: String,
    },
    /// Get the current RTC time and exit
    Get
}

fn main() {
    let args = Args::parse();

    let mut rtc = Ds323x::new_ds3231(I2c::new().expect("Failed to open RTC I2C connection!"));

    let mut mpu = Mpu9250::new(I2c::new().expect("Failed to open MPU I2C connection!"), Delay, Default::default()).expect("Failed to initialize MPU!");

    let mut gps = initialize_gps().expect("Failed to initialize GPS!");
    
    println!("System initialization successful!");
    println!("RTC is: {}", rtc.datetime().expect("Failed to read DateTime from RTC!"));

    if let Some(command) = args.command {
        match command {
            Commands::Clock { command } => match command {
                ClockCommands::Set { time } => {
                    rtc.set_datetime(&NaiveDateTime::parse_from_str(&time, DATE_FORMAT).expect(&format!("Invalid date format! Expected {DATE_FORMAT}")));

                    println!("Clock set! New time is:{}", rtc.datetime().expect("Failed to read DateTime from RTC!"));
                },
                ClockCommands::Get => return,
            },
        }
    }

    if let Err(e) = set_system_clock_from_rtc(&mut rtc) {
        warn!("Failed to set system clock: {}", e.to_string());
    }

    loop {
        gps
            .update(|packet| match packet {
                PacketRef::MonVer(packet) => {
                    println!(
                        "SW version: {} HW version: {}; Extensions: {:?}",
                        packet.software_version(),
                        packet.hardware_version(),
                        packet.extension().collect::<Vec<&str>>()
                    );
                    println!("{:?}", packet);
                },
                PacketRef::NavPvt(sol) => {
                    let has_time = sol.fix_type() == GpsFix::Fix3D
                        || sol.fix_type() == GpsFix::GPSPlusDeadReckoning
                        || sol.fix_type() == GpsFix::TimeOnlyFix;
                    let has_posvel = sol.fix_type() == GpsFix::Fix3D
                        || sol.fix_type() == GpsFix::GPSPlusDeadReckoning;

                    if has_posvel {
                        let pos: Position = (&sol).into();
                        let vel: Velocity = (&sol).into();
                        println!(
                            "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                            pos.lat, pos.lon, pos.alt
                        );
                        println!(
                            "Speed: {:.2} m/s Heading: {:.2} degrees",
                            vel.speed, vel.heading
                        );
                        println!("Sol: {:?}", sol);
                    }

                    if has_time {
                        let time: DateTime<Utc> = (&sol)
                            .try_into()
                            .expect("Could not parse NAV-PVT time field to UTC");
                        println!("Time: {:?}", time);
                    }
                },
                _ => {
                    println!("{:?}", packet);
                },
            })
            .unwrap();
    }
}

#[derive(Debug)]
enum GpsError {
    SerialError(serialport::Error),
}

fn initialize_gps() -> Result<Device, GpsError> {
    let builder = serialport::new("/dev/ttyS0", 9600)
        .flow_control(serialport::FlowControl::None)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .open()
        .map_err(|e| GpsError::SerialError(e))?;

    let mut device = Device::new(builder);

    // Configure the device to talk UBX
    println!("Configuring UART1 port ...");
    device
        .write_all(
            &CfgPrtUartBuilder {
                portid: UartPortId::Uart1,
                reserved0: 0,
                tx_ready: 0,
                mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
                baud_rate: 9600,
                in_proto_mask: InProtoMask::UBLOX,
                out_proto_mask: OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::UBLOX),
                flags: 0,
                reserved5: 0,
            }
            .into_packet_bytes(),
        )
        .expect("Could not configure UBX-CFG-PRT-UART");
    device
        .wait_for_ack::<CfgPrtUart>()
        .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    // Enable the NavPvt packet
    device
        .write_all(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 1, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-NAV-PVT");
    device
        .wait_for_ack::<CfgMsgAllPorts>()
        .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    // Send a packet request for the MonVer packet
    device
        .write_all(&UbxPacketRequest::request_for::<MonVer>().into_packet_bytes())
        .expect("Unable to write request/poll for UBX-MON-VER message");

    // Start reading data
    println!("Opened uBlox device, waiting for messages...");

    Ok(device)
}

// Shamelessly copied https://github.com/ublox-rs/ublox/blob/master/examples/basic_cli/src/main.rs
struct Device {
    port: Box<dyn serialport::SerialPort>,
    parser: Parser<Vec<u8>>,
}

impl Device {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Device {
        let parser = Parser::default();
        Device { port, parser }
    }

    pub fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.port.write_all(data)
    }

    pub fn update<T: FnMut(PacketRef)>(&mut self, mut cb: T) -> std::io::Result<()> {
        loop {
            const MAX_PAYLOAD_LEN: usize = 1240;
            let mut local_buf = [0; MAX_PAYLOAD_LEN];
            let nbytes = self.read_port(&mut local_buf)?;
            if nbytes == 0 {
                break;
            }

            // parser.consume adds the buffer to its internal buffer, and
            // returns an iterator-like object we can use to process the packets
            let mut it = self.parser.consume(&local_buf[..nbytes]);
            loop {
                match it.next() {
                    Some(Ok(packet)) => {
                        cb(packet);
                    },
                    Some(Err(_)) => {
                        // Received a malformed packet, ignore it
                    },
                    None => {
                        // We've eaten all the packets we have
                        break;
                    },
                }
            }
        }
        Ok(())
    }

    pub fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> std::io::Result<()> {
        let mut found_packet = false;
        while !found_packet {
            self.update(|packet| {
                if let PacketRef::AckAck(ack) = packet {
                    if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                        found_packet = true;
                    }
                }
            })?;
        }
        Ok(())
    }

    /// Reads the serial port, converting timeouts into "no data received"
    fn read_port(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        match self.port.read(output) {
            Ok(b) => Ok(b),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    Ok(0)
                } else {
                    Err(e)
                }
            },
        }
    }
}

enum ClockError {
    Fetch,
    CommandStart(String),
    CommandWait(String),
}

impl ToString for ClockError {
    fn to_string(&self) -> String {
        match self {
            Self::Fetch => format!("Failed to fetch clock time"),
            Self::CommandStart(e) => format!("Failed to start date command: {e}"),
            Self::CommandWait(e) => format!("Failed to wait for command: {e}"),
        }
    }
}

fn set_system_clock_from_rtc(rtc: &mut Ds323x<ds323x::interface::I2cInterface<I2c>, ds323x::ic::DS3231>) -> Result<(), ClockError> {
    // Set the system clock
    let time = rtc.datetime().map_err(|_| ClockError::Fetch)?;

    let command = Command::new("date")
        .args(["-s", &time.format(DATE_FORMAT).to_string()])
        .output()
        .map_err(|e| ClockError::CommandStart(e.to_string()));

    match command {
        Ok(code) => if !code.status.success() {
            Err(ClockError::CommandWait(format!("Bad exit code: {}", code.status.to_string())))
        } else {
            Ok(())
        },
        Err(e) => Err(ClockError::CommandWait(e.to_string())),
    }
}
