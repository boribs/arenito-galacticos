use crate::arenito::*;
use bevy::prelude::*;
use rand::{prelude::thread_rng, Rng};

/// This struct's purpose is to generalize "Error Vector" generation.
/// Since this is a needed step on simulating sensors.
struct SensorError;
impl SensorError {
    const DEFAULT_MIN: f32 = -0.1;
    const DEFAULT_MAX: f32 = 0.1;

    /// Returns a Vec3 with random values in the range min..max.
    pub fn vec(min: f32, max: f32) -> Vec3 {
        Vec3::splat(thread_rng().gen_range(min..max))
    }

    /// Returns the default (kinda like std::default::Default) vector
    /// with an error range DEFAULT_MIN..DEFAULT_MAX.
    pub fn default() -> Vec3 {
        SensorError::vec(SensorError::DEFAULT_MIN, SensorError::DEFAULT_MAX)
    }
}

/// This struct is responsible for the simulations of all sensors related to Arenito.
/// There's currently only an implementation for an Accelerometer. A Gyroscope is on the way.
pub struct Sensor;

impl Sensor {
    const ACCELERATION_MAX: f32 = 9.8; // 1g!

    /// Gets Arenito's "real" acceleration and converts it
    /// to something the real accelerometer would return:
    /// A value between 0 and 1024 that represents the magnitude
    /// of the acceleration on each axis.
    pub fn read_acc(arenito: &Arenito) -> Vec3 {
        // This is the upper bound for the accelerometer readings.
        // Depends entirely on the hardware.

        // get acceleration value
        // convert to absolute value
        let mut acc = arenito.acc.abs();

        // add error
        acc += SensorError::default();

        // interpolate each value between [0, 1024],
        // considering that Sensor::ACCELERATION_MAX maps to 1024.
        acc.x = 1024.0 * (acc.x / Sensor::ACCELERATION_MAX);
        acc.y = 1024.0 * (acc.y / Sensor::ACCELERATION_MAX);
        acc.z = 1024.0 * (acc.z / Sensor::ACCELERATION_MAX);

        acc
    }

        acc.abs()
    }
}
