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
        let mut rng = thread_rng();
        Vec3::new(
            rng.gen_range(min..max),
            rng.gen_range(min..max),
            rng.gen_range(min..max),
        )
    }

    /// Returns the default (kinda like std::default::Default) vector
    /// with an error range DEFAULT_MIN..DEFAULT_MAX.
    pub fn default() -> Vec3 {
        SensorError::vec(SensorError::DEFAULT_MIN, SensorError::DEFAULT_MAX)
    }
}

/// This struct is responsible for the simulations of all sensors related to Arenito.
/// This sensor simulation is based on the MPU6050 Chip, which includes an acceleromter
/// and a gyroscope.
/// The outputs of this simulator are trying to be as similar as posible to this:
/// (https://randomnerdtutorials.com/arduino-mpu-6050-accelerometer-gyroscope/)
pub struct MPU6050;

impl MPU6050 {
    // This is the upper bound for the accelerometer readings.
    // Depends entirely on the hardware.
    // Acceleration reads CAN NOT be higher than this.
    const ACCELERATION_MAX: f32 = 9.8; // 1g!

    /// Gets Arenito's "real" acceleration and converts it
    /// to something the real accelerometer would return:
    /// A value between 0 and 1024 that represents the magnitude
    /// of the acceleration on each axis.
    pub fn read_acc(arenito: &Arenito) -> Vec3 {
        // get acceleration value
        // convert to absolute value
        let mut acc = arenito.acc.abs();

        // add error
        acc += SensorError::default();

        // interpolate each value between [0, 1024],
        // considering that Sensor::ACCELERATION_MAX maps to 1024.
        acc.abs() * 1024.0 / MPU6050::ACCELERATION_MAX
    }

    /// Gets Arenito's "real" rotation.
    /// Technically the sensor outputs rotational speed, but I'm
    /// too lazy to simulate that.
    /// This implementation skips all the math needed to convert
    /// from rotational speed to "current rotation" altogether.
    pub fn read_rot(arenito: &Arenito) -> Vec3 {
        arenito.rot + SensorError::default() * 2.0
    }
}

#[cfg(test)]
mod sensor_read_tests {
    use super::*;

    fn acc_within_value(vec: &Vec3) {
        // println!("{}", vec);
        assert!(vec.x >= 0.0 && vec.x <= 1024.0);
        assert!(vec.y >= 0.0 && vec.y <= 1024.0);
        assert!(vec.z >= 0.0 && vec.z <= 1024.0);
    }

    #[test]
    fn sensor_acc_reads_dont_go_to_negative_values() {
        let mut rng = thread_rng();
        let mut arenito = Arenito::new();

        for _ in 0..100 {
            arenito.acc = Vec3::new(
                rng.gen_range(-2.1..2.1),
                rng.gen_range(-2.1..2.1),
                rng.gen_range(-2.1..2.1),
            );
            let read = MPU6050::read_acc(&arenito);
            acc_within_value(&read);
        }
    }

    // No idea how to or what to test for gyro reads...
}
