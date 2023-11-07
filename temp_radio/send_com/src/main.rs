mod sender_info;
mod data;
mod util;

use data::TelData;
use sender_info::{send_command, send_data};
use util::calculate_checksum;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task = 1; // 1 - send com, 2 = send teldata

    if task == 1 {

        let digit = "10";
        send_command(digit.to_string())?; 

    } else {

        let telemetry_data = TelData {

            temp : "75".to_string(),           
            gps_x :"x_cor".to_string(),
            gps_y : "y_cor".to_string(),
            gps_z : "y_cor".to_string(),
            acc_x : "x_acc".to_string(),
            acc_y : "y_acc".to_string(),
            acc_z : "z_acc".to_string(),
        };

        send_data(&telemetry_data)?;

    }
    Ok(())
}