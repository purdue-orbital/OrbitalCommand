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
pub enum MessageToLaunch {
    Abort,
    Launch,
    Cut,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageToGround {
    ImuTelemetry {
        temperature: f64,
        acceleration: Vec3,
        gyro: Vec3,
    },
    GpsTelemetry {
        altitude: f64,
        latitude: f64,
        longitude: f64,
        velocity: f64,
        heading: f64,
    },
}

impl TryFrom<&[u8]> for MessageToGround {
    type Error = bincode::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}

impl TryFrom<MessageToGround> for Vec<u8> {
    type Error = bincode::Error;

    fn try_from(value: MessageToGround) -> Result<Self, Self::Error> {
        bincode::serialize(&value)
    }
}

impl TryFrom<&[u8]> for MessageToLaunch {
    type Error = bincode::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}

impl TryFrom<MessageToLaunch> for Vec<u8> {
    type Error = bincode::Error;

    fn try_from(value: MessageToLaunch) -> Result<Self, Self::Error> {
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
