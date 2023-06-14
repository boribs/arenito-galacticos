use bevy::prelude::*;

const ROTATION_SPEED: f32 = 1.0; // rad x sec
const MOVEMENT_ACCEL: f32 = 1.0; // units x sec

#[derive(Resource)]
pub struct Arenito {
    center: Vec3,
    vel: Vec3,
    acc: Vec3,
    look_angle: f32, // on the y axis
}

impl Arenito {
    pub fn new() -> Self {
        Arenito {
            center: Vec3::ZERO,
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            look_angle: 0.0,
        }
    }

    /// Sets the acceleration to "advance acceleration".
    pub fn forward(&mut self) {
        todo!("forward movement");
    }

    /// Sets Arenito in "rotation mode" - sets the rotation acceleration
    /// to the correct values.
    pub fn rotate(&mut self) {
        todo!("rotation (tank controls)");
    }

    /// Applies the movement given some delta time.
    pub fn update(&mut self, delta_time: u32) {
        todo!("movement updates!");
    }
}
