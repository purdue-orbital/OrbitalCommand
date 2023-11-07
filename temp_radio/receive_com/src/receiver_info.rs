use std::io::Read;
use std::net::TcpListener;
use serde_json;
use std::thread;
use std::sync::{Arc, Mutex};

use crate::data::GROUND;
use crate::data::ROCKET;
use crate::data::TelData;
use crate::calculate_checksum;

pub fn receiving_command () -> Result<(), std::io::Error> {   //rocket will run this function

    let functions: Arc<Mutex<Vec<fn()>>> = Arc::new(Mutex::new(vec![cut, release, abort]));

    let length = {
        let functions = functions.lock().unwrap();
        functions.len()
    };
    //println!("{}", length);
  //  let functions: Vec<fn()> = vec![cut, release, abort];
    
    let receiving_ip = ROCKET; //own ip

    let listener = TcpListener::bind(receiving_ip)?;

    println!("Waiting for a connection...");

    for stream in listener.incoming() {

        match stream {

            Ok(mut socket) => {
                 
                let mut buffer = vec![];
                if let Err(e) = socket.read_to_end(&mut buffer) {
                    eprintln!("Error in reading data: { }", e);
                }
                else if buffer.len() >= 2 {
                    //println!("{}", buffer.len());
                    let (command, checksum) = buffer.split_at(buffer.len() - 4);
                    let calculated_checksum = calculate_checksum(command);
                    
                    //let checksum_u32 = u32::from_le_bytes([checksum[0], checksum[1], checksum[2], checksum[3]]);
                    //println!("{:08X}", checksum_u32);
            
                    if checksum == calculated_checksum {
                        
                        if let Ok(digit_str) = std::str::from_utf8(command) {

                            if let Ok(digit) = digit_str.parse::<isize>() {
                    
                                if digit >= 0 && digit < length as isize {
                                    let digit_us: usize = digit.try_into().unwrap();
                                    let functions_clone = Arc::clone(&functions);
                                    thread::spawn(move || {
                                        let functions = functions_clone.lock().unwrap();
                                        if let Some(f) = functions.get(digit_us) {
                                            f();
                                        }
                                        });

                                    } else {
                                    eprintln!("Index is out of range.");
                                    }
                       


                            } else {
                                eprintln!("Invalid digit format.");
                            }
                        } else {
                            eprintln!("Received data is not valid UTF-8.");
                        }


                    }else {
                        eprintln!("Checksum mismatch. Data may be corrupted.");
                    }
                } else {
                      println!("{}", buffer.len());
                      eprintln!("Received data is too short for checksum verification.");

                   }

            }

           Err(e) => {

               eprintln!("Error accepting connection: {} ", e);   

           }

       }

   }  

    Ok(())    
}

fn cut() {
    println!("Cut function executed.");
}

fn release() {
    println!("Release function executed.");
}

fn abort() {
    println!("Abort function executed.");
}


fn handle_data (telemetry_data : TelData) {
    println!("Received telemetry data:");
    println!("Field1: {}", telemetry_data.temp);
    println!("Field2: {}", telemetry_data.gps_x);
    println!("Field3: {}", telemetry_data.gps_x);
    println!("Field4: {}", telemetry_data.gps_z);
    println!("Field5: {}", telemetry_data.acc_x);
    println!("Field6: {}", telemetry_data.acc_y);
    println!("Field7: {}", telemetry_data.acc_z);

}

pub fn receiving_teldata () -> Result<(), std::io::Error> { //ground station will run this
    
    let receiving_ip = GROUND;

    let listener = TcpListener::bind(receiving_ip)?;
    
    println!("Waiting for a connection...");

    for stream in listener.incoming() {

        if let Ok(mut stream) = stream {
            let mut raw_data = Vec::new();
            if let Err(e) = stream.read_to_end(&mut raw_data) {
                eprintln!("Error in reading data: { }", e);
                continue
            }

            if raw_data.len() >= 2 {
                let (telemetry_data, checksum) = raw_data.split_at(raw_data.len() - 4);
                let calculated_checksum = calculate_checksum(telemetry_data);

                //let checksum_u32 = u32::from_le_bytes([checksum[0], checksum[1], checksum[2], checksum[3]]);
                //println!("{:08X}", checksum_u32);
                if checksum == calculated_checksum {

                    if let Ok(deserialized) = serde_json::from_slice(telemetry_data) {
                        thread::spawn(move || {handle_data(deserialized); });
                    } else {
                        eprintln!("Error in deserializing telemetry data.");
                    }
                } else {
                    eprintln!("Checksum mismatch. Data may be corrupted.");
                }
            } else {
                eprintln!("Received data is too short for checksum verification.");
            }
        } else {
            eprintln!("Error accepting connection.");
        }
    }
    Ok(())
}

//sudo ufw allow 12345/tcp 
//12345 is your port num

//checksums?
