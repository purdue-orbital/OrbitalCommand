#![deny(clippy::unwrap_used)]

use std::{
    io::ErrorKind,
    ops::DerefMut,
    process::Command,
    sync::{atomic::AtomicBool, mpsc::channel, Arc, Mutex, RwLock},
    thread::{self, sleep},
    time::{Duration as StdDuration, Instant},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{Parser as ClapParser, Subcommand};
use common::{MessageToGround, MessageToLaunch, Vec3};
use ds323x::{DateTimeAccess, Ds323x};
use flexi_logger::{detailed_format, FileSpec, Logger};
use log::{debug, info, warn};
use radio::RadioStream;
use rolly::Mpu9250;
use rppal::{gpio::Gpio, gpio::OutputPin, hal::Delay, i2c::I2c};
use signal_hook::{consts::TERM_SIGNALS, flag};
use ublox::*;
use ublox::{
    CfgPrtUartBuilder, DataBits, InProtoMask, OutProtoMask, Parity, StopBits, UartMode, UartPortId,
};

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
    },
}

#[derive(Debug, Subcommand)]
enum ClockCommands {
    /// Set the current RTC time with an ISO8601 string (%Y-%m-%d %H:%M:%S) and exit
    Set { time: String },
    /// Get the current RTC time and exit
    Get,
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

    let (file_log, _fl_handle) = Logger::try_with_env_or_str("debug")
        .unwrap()
        .log_to_file(FileSpec::default().directory("/var/log/orbital"))
        .format(detailed_format)
        .rotate(
            flexi_logger::Criterion::AgeOrSize(flexi_logger::Age::Day, 20_000_000),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(100),
        )
        .write_mode(flexi_logger::WriteMode::Direct)
        .build()
        .unwrap();

    let master_log = multi_log::MultiLogger::new(vec![stdout_log, file_log]);
    log::set_boxed_logger(Box::new(master_log));

    let termination_flag = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&termination_flag))
            .expect("Failed to register conditional signal!");
        // But this will "arm" the above for the second time, by setting it to true.
        // The order of registering these is important, if you put this one first, it will
        // first arm and then terminate ‒ all in the first round.
        flag::register(*sig, Arc::clone(&termination_flag)).expect("Failed to register signal!");
    }
    info!("Signals hooked");

    let args = Args::parse();

    let mut rtc = Ds323x::new_ds3231(I2c::new().expect("Failed to open RTC I2C connection!"));
    rtc.enable().expect("RTC enable to work all the time");

    info!(
        "RTC initialized. Time reads: {}",
        rtc.datetime().expect("Failed to read DateTime from RTC!")
    );

    // let mut mpu = Mpu9250::new(I2c::new().expect("Failed to open MPU I2C connection!"), Delay, Default::default()).expect("Failed to initialize MPU!");
    // mpu.init().unwrap();
    let mut mpu = Mpu9250::marg_default(
        I2c::new().expect("Failed to open MPU I2C connection!"),
        &mut Delay,
    )
    .expect("unable to make MPU9250");

    let mut gps = initialize_gps().expect("Failed to initialize GPS!");

    info!("System initialization successful!");

    if let Some(command) = args.command {
        match command {
            Commands::Clock { command } => match command {
                ClockCommands::Set { time } => {
                    let parsed = NaiveDateTime::parse_from_str(&time, DATE_FORMAT)
                        .expect(&format!("Invalid date format! Expected {DATE_FORMAT}"));
                    info!("Parsed format: {}", parsed);
                    rtc.set_datetime(&parsed);

                    info!(
                        "Clock set! New time is: {}",
                        rtc.datetime().expect("Failed to read DateTime from RTC!")
                    );
                    return;
                }
                ClockCommands::Get => return,
            },
        }
    }

    if let Err(e) = set_system_clock_from_rtc(&mut rtc) {
        warn!("Failed to set system clock: {}", e.to_string());
    }

    info!("System clock set from RTC! UTC is now {:?}", Utc::now());

    // Just turn these off for now
    let gpio = Gpio::new().unwrap();
    let mut ign = gpio.get(IGNITITON_BCM_GPIO).unwrap().into_output();
    ign.set_low();
    let mut qdm = gpio.get(QDM_BCM_GPIO).unwrap().into_output();
    qdm.set_low();

    let rip_done = Arc::new(AtomicBool::new(false));
    let rip_pin = Arc::new(Mutex::new(
        Gpio::new()
            .unwrap()
            .get(RIP_BCM_GPIO)
            .unwrap()
            .into_output(),
    ));
    rip_pin.lock().unwrap().set_low();
    let rip_pin_gps = rip_pin.clone();

    // TODO: Does this need USB-3?
    // let radio = Arc::new(RwLock::new(RadioStream::new().unwrap()));

    let (msg_tx, msg_rx) = channel();

    let tf_gps = termination_flag.clone();
    let tx_gps = msg_tx.clone();
    let rip_done_gps = rip_done.clone();
    let gps_hnd = thread::spawn(move || {
        let mut last_packet = Instant::now();
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
                        let has_time = sol.fix_type() == GpsFix::Fix3D
                            || sol.fix_type() == GpsFix::GPSPlusDeadReckoning
                            || sol.fix_type() == GpsFix::TimeOnlyFix;
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
                            info!("TELEMETRY: Lat {:.5} Long {:5} Alt {:.2} m Spd {:.2} m/s Head {:.2} deg", pos.lat, pos.lon, pos.alt, vel.speed, vel.heading);
                            // println!("Sol: {:?}", sol);
                            
                            // Dead man's switch at 10,000 ft
                            last_packet = Instant::now();
                            if pos.alt >= 3_048.0 && !rip_done_gps.load(std::sync::atomic::Ordering::SeqCst) {
                                info!("Dead man's switch hit! Cutting down.");
                                rip_done_gps.store(true, std::sync::atomic::Ordering::SeqCst);
                                cutdown(&mut rip_pin_gps.lock().unwrap());
                            }
                            
                            tx_gps.send(MessageToGround::GpsTelemetry { altitude: pos.alt, latitude: pos.lat, longitude: pos.lon, velocity: vel.speed, heading: vel.heading }).unwrap();
                        }
    
                        if has_time {
                            let time: DateTime<Utc> = (&sol)
                                .try_into()
                                .expect("Could not parse NAV-PVT time field to UTC");
                            info!("GPS TIME (UTC): {:?}", time);
                        }
                    },
                    _ => {
                        println!("{:?}", packet);
                    },
                })
                .unwrap();

            if Instant::now() > last_packet + StdDuration::from_millis(600 * 1000)
                && !rip_done_gps.load(std::sync::atomic::Ordering::SeqCst)
            {
                info!("No GPS for over 10 minutes! Cutting down.");
                // rip_done_gps.store(true, std::sync::atomic::Ordering::SeqCst);
                // cutdown(&mut rip_pin_gps.lock().unwrap());
            }
        }
    });

    let tf_mpu = termination_flag.clone();
    let tx_mpu = msg_tx.clone();
    let mpu_hnd = thread::spawn(move || {
        while !tf_mpu.load(std::sync::atomic::Ordering::SeqCst) {
            let all: rolly::MargMeasurements<[f32; 3]> = mpu.all().unwrap();
            tx_mpu.send(MessageToGround::ImuTelemetry {
                temperature: all.temp as f64,
                acceleration: Vec3 {
                    x: all.accel[0] as f64,
                    y: all.accel[1] as f64,
                    z: all.accel[2] as f64,
                },
                gyro: Vec3 {
                    x: all.gyro[0] as f64,
                    y: all.gyro[1] as f64,
                    z: all.gyro[2] as f64,
                },
            });

            sleep(StdDuration::from_millis(1_000));
        }
    });

    drop(msg_tx);

    let rip_pin_rx = rip_pin.clone();
    let rip_done_rx = rip_done.clone();
    // let radio_rx = radio.clone();
    let tf_radio = termination_flag.clone();
    let radio_hnd = thread::spawn(move || {
        // while !tf_radio.load(std::sync::atomic::Ordering::SeqCst) {
        //     let received = radio_rx.read().unwrap().read().unwrap();
        //
        //     if let Ok(val) = MessageToLaunch::try_from(received.as_slice()) {
        //         match val {
        //             MessageToLaunch::Abort => todo!("abort"),
        //             MessageToLaunch::Launch => todo!("launch"),
        //             MessageToLaunch::Cut => if !rip_done_rx.load(std::sync::atomic::Ordering::SeqCst) {
        //                 cutdown(rip_pin_rx.lock().unwrap().deref_mut());
        //                 rip_done_rx.store(true, std::sync::atomic::Ordering::SeqCst);
        //             },
        //         }
        //     }
        //
        //     thread::sleep(StdDuration::from_millis(100));
        // }
    });

    let rip_pin_timer = rip_pin.clone();
    let tf_timer = termination_flag.clone();
    let start = Instant::now();
    let rip_done_timer = rip_done.clone();
    let timer_hnd = thread::spawn(move || {
        while !tf_timer.load(std::sync::atomic::Ordering::SeqCst) {
            // Timed cutdown
            if Instant::now() > start + StdDuration::from_millis(CUTDOWN_TIME_SECS * 1000)
                && !rip_done_timer.load(std::sync::atomic::Ordering::SeqCst)
            {
                info!("Time's up! Cutting down.");
                rip_done_timer.store(true, std::sync::atomic::Ordering::SeqCst);
                cutdown(&mut rip_pin_timer.lock().unwrap());
            }

            thread::sleep(StdDuration::from_millis(100));
        }
    });

    // let radio_tx = radio.clone();
    while !termination_flag.load(std::sync::atomic::Ordering::SeqCst) {
        for msg in msg_rx.iter() {
            info!("Generated radio message: {:?}", msg);
            let msg: Vec<u8> = msg.try_into().unwrap();
            // radio_tx.read().unwrap().transmit(&msg).unwrap();
        }
    }

    mpu_hnd.join().unwrap();
    gps_hnd.join().unwrap();
    radio_hnd.join().unwrap();
}

const CUTDOWN_TIME_SECS: u64 = 500;

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
        .write_with_ack::<CfgPrtUart>(
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
    // device
    //     .wait_for_ack::<CfgPrtUart>()
    //     .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

    // Enable the NavPvt packet
    device
        .write_with_ack::<CfgMsgAllPorts>(
            &CfgMsgAllPortsBuilder::set_rate_for::<NavPvt>([0, 1, 0, 0, 0, 0]).into_packet_bytes(),
        )
        .expect("Could not configure ports for UBX-NAV-PVT");
    // device
    //     .wait_for_ack::<CfgMsgAllPorts>()
    //     .expect("Could not acknowledge UBX-CFG-PRT-UART msg");

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

const UART_TIMEOUT: StdDuration = StdDuration::from_millis(1000);
const UART_RETRIES: usize = 10;

impl Device {
    pub fn new(port: Box<dyn serialport::SerialPort>) -> Device {
        let parser = Parser::default();
        Device { port, parser }
    }

    pub fn write_with_ack<T: UbxPacketMeta>(&mut self, data: &[u8]) -> std::io::Result<()> {
        // First write the packet
        self.write_all(data)?;

        // Start waiting for ACK, allowing retries until max is hit
        let mut attempts = 0;
        while attempts < UART_RETRIES {
            match self.wait_for_ack::<T>() {
                Ok(()) => return Ok(()),
                Err(e) => match e.kind() {
                    ErrorKind::TimedOut => {
                        // Didn't receive packet, try again
                        self.write_all(data)?;
                        attempts += 1;
                    }
                    _ => return Err(e),
                },
            }
        }

        Err(std::io::Error::from(ErrorKind::TimedOut))
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
                    }
                    Some(Err(_)) => {
                        // Received a malformed packet, ignore it
                    }
                    None => {
                        // We've eaten all the packets we have
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn wait_for_ack<T: UbxPacketMeta>(&mut self) -> std::io::Result<()> {
        let mut found_packet = false;
        let start = Instant::now();
        while !found_packet {
            self.update(|packet| {
                if let PacketRef::AckAck(ack) = packet {
                    if ack.class() == T::CLASS && ack.msg_id() == T::ID {
                        found_packet = true;
                    }
                }
            })?;

            if start.elapsed() > UART_TIMEOUT && !found_packet {
                return Err(std::io::Error::from(ErrorKind::TimedOut));
            }
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
            }
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

fn set_system_clock_from_rtc(
    rtc: &mut Ds323x<ds323x::interface::I2cInterface<I2c>, ds323x::ic::DS3231>,
) -> Result<(), ClockError> {
    // Set the system clock
    let time = rtc.datetime().map_err(|_| ClockError::Fetch)?;

    let command = Command::new("date")
        .args(["-s", &time.format(DATE_FORMAT).to_string()])
        .output()
        .map_err(|e| ClockError::CommandStart(e.to_string()));

    match command {
        Ok(code) => {
            if !code.status.success() {
                Err(ClockError::CommandWait(format!(
                    "Bad exit code: {}",
                    code.status.to_string()
                )))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(ClockError::CommandWait(e.to_string())),
    }
}

const RIP_BCM_GPIO: u8 = 5;
const IGNITITON_BCM_GPIO: u8 = 6;
const QDM_BCM_GPIO: u8 = 13;

fn cutdown(rip_pin: &mut OutputPin) {
    rip_pin.set_high();
    sleep(StdDuration::from_millis(5000));
    rip_pin.set_low();
}
