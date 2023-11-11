use std::io::Read;
use serde_json;
use std::thread;
use std::sync::{Arc, Mutex};

use crate::data::GROUND;
use crate::data::ROCKET;
use crate::data::TelData;
use crate::calculate_checksum;

extern crate radio;
use radio::RadioStream;

pub fn receiving_command () -> Result<(), std::io::Error> {   //rocket will run this function

    let functions: Arc<Mutex<Vec<fn()>>> = Arc::new(Mutex::new(vec![cut, release, abort]));

    let length = {

        let functions = functions.lock().unwrap();

        functions.len()
    };
 
    println!("Waiting for a connection...");

    while(1) {

    let stream = RadioStream::read().map_err(|e| Error::new(ErrorKind::Other, format!("Custom error: {}", e)));
    
    match stream {

        Ok(buffer) => {

            if buffer.len() >= 2 {

                let (command, checksum) = buffer.split_at(buffer.len() - 4);

                let calculated_checksum = calculate_checksum(command);
        
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

    println!("Waiting for a connection...");

    while(1) {

        let stream = RadioStream::read().map_err(|e| Error::new(ErrorKind::Other, format!("Custom error: {}", e)));
        
        match stream {

            Ok(stream) {

                if stream.len() >= 2 {

                    let (telemetry_data, checksum) = stream.split_at(stream.len() - 4);

                    let calculated_checksum = calculate_checksum(telemetry_data);

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

            } Err(e) => {

                eprintln!("Error accepting connection. {}", e);

            }

        }

    }
    Ok(())
}

