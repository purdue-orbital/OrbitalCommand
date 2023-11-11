#![deny(clippy::unwrap_used)]

use std::{process::Command, str::FromStr, sync::{Arc, atomic::AtomicBool, mpsc::channel}, thread::{self, sleep}, time::Duration as StdDuration};

use chrono::{NaiveDateTime, Utc, DateTime, Duration};
use common::{MessageToGround, Vec3};
use ds323x::{Ds323x, DateTimeAccess};
use flexi_logger::{Logger, FileSpec, detailed_format};
use mpu9250_i2c::Mpu9250;
use rppal::{i2c::I2c, hal::Delay};
use serialport::SerialPort;
use signal_hook::{consts::TERM_SIGNALS, flag};
use ublox::{CfgPrtUartBuilder, UartPortId, UartMode, DataBits, Parity, StopBits, InProtoMask, OutProtoMask};
use clap::{Parser as ClapParser, Subcommand};
use log::{warn, error, info, debug};
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
    // Enable logging
    let (stdout_log, _stdout_handle) = Logger::try_with_str("debug")
        .unwrap()
        .log_to_stdout()
        .format(detailed_format)
        .write_mode(flexi_logger::WriteMode::Direct)
        .build()
        .unwrap();

    let (file_log, _fl_handle) = Logger::try_with_env_or_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().directory("/var/log/orbital"))
        .format(detailed_format)
        .rotate(flexi_logger::Criterion::AgeOrSize(flexi_logger::Age::Day, 20_000_000), flexi_logger::Naming::Timestamps, flexi_logger::Cleanup::KeepLogFiles(100))
        .write_mode(flexi_logger::WriteMode::Direct).build().unwrap();

    let master_log = multi_log::MultiLogger::new(vec![stdout_log, file_log]);
    log::set_boxed_logger(Box::new(master_log));

    let termination_flag = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&termination_flag)).expect("Failed to register conditional signal!");
        // But this will "arm" the above for the second time, by setting it to true.
        // The order of registering these is important, if you put this one first, it will
        // first arm and then terminate ‒ all in the first round.
        flag::register(*sig, Arc::clone(&termination_flag)).expect("Failed to register signal!");
    }
    info!("Signals hooked");

    let args = Args::parse();

    let mut rtc = Ds323x::new_ds3231(I2c::new().expect("Failed to open RTC I2C connection!"));

    let mut mpu = Mpu9250::new(I2c::new().expect("Failed to open MPU I2C connection!"), Delay, Default::default()).expect("Failed to initialize MPU!");
    mpu.init().unwrap();

    let mut gps = initialize_gps().expect("Failed to initialize GPS!");
    
    info!("System initialization successful!");
    info!("RTC is: {}", rtc.datetime().expect("Failed to read DateTime from RTC!"));

    if let Some(command) = args.command {
        match command {
            Commands::Clock { command } => match command {
                ClockCommands::Set { time } => {
                    rtc.set_datetime(&NaiveDateTime::parse_from_str(&time, DATE_FORMAT).expect(&format!("Invalid date format! Expected {DATE_FORMAT}")));

                    info!("Clock set! New time is:{}", rtc.datetime().expect("Failed to read DateTime from RTC!"));
                },
                ClockCommands::Get => return,
            },
        }
    }

    if let Err(e) = set_system_clock_from_rtc(&mut rtc) {
        warn!("Failed to set system clock: {}", e.to_string());
    }

    let (msg_tx, msg_rx) = channel();

    let tf_gps = termination_flag.clone();
    let tx_gps = msg_tx.clone();
    let gps_hnd = thread::spawn(move || {
        while !tf_gps.load(std::sync::atomic::Ordering::SeqCst) {
            gps
                .update(|packet| match packet {
                    PacketRef::MonVer(packet) => {
                        debug!(
                            "SW version: {} HW version: {}; Extensions: {:?}",
                            packet.software_version(),
                            packet.hardware_version(),
                            packet.extension().collect::<Vec<&str>>()
                        );
                        debug!("{:?}", packet);
                    },
                    PacketRef::NavPvt(sol) => {
                        // let has_time = sol.fix_type() == GpsFix::Fix3D
                        //     || sol.fix_type() == GpsFix::GPSPlusDeadReckoning
                        //     || sol.fix_type() == GpsFix::TimeOnlyFix;
                        let has_posvel = sol.fix_type() == GpsFix::Fix3D
                            || sol.fix_type() == GpsFix::GPSPlusDeadReckoning;
    
                        if has_posvel {
                            let pos: Position = (&sol).into();
                            let vel: Velocity = (&sol).into();
                            // println!(
                            //     "Latitude: {:.5} Longitude: {:.5} Altitude: {:.2}m",
                            //     pos.lat, pos.lon, pos.alt
                            // );
                            // println!(
                            //     "Speed: {:.2} m/s Heading: {:.2} degrees",
                            //     vel.speed, vel.heading
                            // );
                            // println!("Sol: {:?}", sol);
                            
                            tx_gps.send(MessageToGround::GpsTelemetry { altitude: pos.alt, latitude: pos.lat, longitude: pos.lon, velocity: vel.speed, heading: vel.heading }).unwrap();
                        }
    
                        // if has_time {
                        //     let time: DateTime<Utc> = (&sol)
                        //         .try_into()
                        //         .expect("Could not parse NAV-PVT time field to UTC");
                        //     println!("Time: {:?}", time);
                        // }
                    },
                    _ => {
                        println!("{:?}", packet);
                    },
                })
                .unwrap();
        }
    });

    let tf_mpu = termination_flag.clone();
    let tx_mpu = msg_tx.clone();
    let mpu_hnd = thread::spawn(move || {
        while !tf_mpu.load(std::sync::atomic::Ordering::SeqCst) {
            let acc = mpu.get_accel().unwrap();
            tx_mpu.send(MessageToGround::ImuTelemetry { temperature: mpu.get_temperature_celsius().unwrap() as f64, acceleration: Vec3 {
                x: acc.x as f64,
                y: acc.y as f64,
                z: acc.z as f64,
            } });

            sleep(StdDuration::from_millis(1_000));
        }
    });

    drop(msg_tx);

    while !termination_flag.load(std::sync::atomic::Ordering::SeqCst) {
        for msg in msg_rx.iter() {
            println!("Message received: {:?}", msg);
        }
    }

    mpu_hnd.join();
    gps_hnd.join();
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
    debug!("Configuring UART1 port ...");
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
    // TODO: Make retries automatic
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
    debug!("Opened uBlox device, waiting for messages...");

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
