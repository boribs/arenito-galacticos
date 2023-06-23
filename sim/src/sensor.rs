use crate::arenito::*;
use bevy::prelude::*;
use rand::{prelude::thread_rng, Rng};

/// This struct is responsible for the simulations of all sensors related to Arenito.
/// There's currently only an implementation for an Accelerometer. A Gyroscope is on the way.
pub struct Sensor;

impl Sensor {
    /// Gets Arenito's "real" acceleration and converts it
    /// to something the real accelerometer would return:
    /// A value between 0 and 1024 that represents the magnitude
    /// of the acceleration on each axis.
    pub fn read_acc(arenito: &Arenito) -> Vec3 {
        // This is the upper bound for the accelerometer readings.
        // Depends entirely on the hardware.
        const ACCELERATION_MAX: f32 = 9.8; // 1g!
        const ERR_MIN: f32 = -0.1;
        const ERR_MAX: f32 = 0.1;

        // get acceleration value
        // convert to absolute value
        let mut acc = arenito.acc.abs();

        // add error
        acc += Vec3::splat(thread_rng().gen_range(ERR_MIN..ERR_MAX));

        // interpolate each value between [0, 1024],
        // considering that Sensor::ACCELERATION_MAX maps to 1024.
        acc.x = 1024.0 * (acc.x / ACCELERATION_MAX);
        acc.y = 1024.0 * (acc.y / ACCELERATION_MAX);
        acc.z = 1024.0 * (acc.z / ACCELERATION_MAX);

        acc.abs()
    }
}
