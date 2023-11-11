use std::io::Write;
use std::net::TcpStream;

use crate::data::GROUND;
use crate::data::ROCKET;
use crate::data::TelData;
use crate::calculate_checksum;

extern crate radio;
use radio::RadioStream

pub fn send_command (command_enum : String) -> Result<(), std::io::Error> {  //for ground station to send command

    let receiving_ip = ROCKET;  //rocket ip

    let command_bytes = command_enum.as_bytes();

    // Calculate the checksum for the command bytes
    let checksum = calculate_checksum(command_bytes);

    let checksum_u32 = u32::from_le_bytes(checksum);
   // println!("{:08X}", checksum_u32);

    // Create a Vec to hold the data, including the command and checksum
    let mut data_with_checksum = Vec::with_capacity(command_bytes.len() + 4);

    data_with_checksum.extend_from_slice(command_bytes);

    data_with_checksum.extend(&checksum);

    RadioStream::transmit(data_with_checksum)?;

    //let mut stream = TcpStream::connect(receiving_ip)?;

    //stream.write_all(&data_with_checksum)?;

    Ok(())
}


pub fn send_data (telemetry_data : &TelData) -> Result<(), std::io::Error> {  //for rocket to send telemetry data
    
    let serialized_data = serde_json::to_string(telemetry_data)?;

    let ground_ip = GROUND;

    let data_bytes = serialized_data.as_bytes();
    
    let checksum = calculate_checksum(data_bytes);

    let mut data_with_checksum = Vec::with_capacity(data_bytes.len() + 4);

    data_with_checksum.extend_from_slice(data_bytes);

    data_with_checksum.extend(&checksum);

    RadioStream::transmit(data_with_checksum)?;

    //let mut stream = TcpStream::connect(ground_ip)?;

    //stream.write_all(&data_with_checksum)?;

    Ok(())
}
   
