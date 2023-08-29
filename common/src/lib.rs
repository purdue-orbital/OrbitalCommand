#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Abort,
    Launch,
    Cut,
    Update,
    Telemetry {
        temperature: f64,
        gps: Vec3,
        acceleration: Vec3,
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = bincode::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}

impl TryFrom<Message> for Vec<u8> {
    type Error = bincode::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        bincode::serialize(&value)
    }
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
