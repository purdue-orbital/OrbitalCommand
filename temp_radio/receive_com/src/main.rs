
mod receiver_info;
mod data;
mod util;

use receiver_info::{receiving_command, receiving_teldata};
use util::calculate_checksum;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task = 1; //1 - receive command, 2 - receive tel data

    if task == 1 {
        receiving_command()?;
    } else {
        receiving_teldata()?;
    }

    Ok(())
}