use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TelData {
    pub temp : String,
    pub gps_x : String,
    pub gps_y : String,
    pub gps_z : String,
    pub acc_x : String,
    pub acc_y : String,
    pub acc_z : String,
 }

//pub const ROCKET: &str = "127.0.0.1:2040";
//pub const GROUND: &str = "127.0.0.1:2040";