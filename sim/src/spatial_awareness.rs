use bevy::prelude::*;
use crate::sensor::MPU6050;
use crate::wire::*;
use crate::arenito::*;

/// This struct is used when calculating how much Arenito has moved
/// since the last frame, as a means of storing some values needed
/// for the calculation.
#[derive(Resource)]
pub struct CalculatedMovement {
    pub acc: Vec3,
    pub vel: Vec3,
    pub pos: Vec3,
}

impl CalculatedMovement {
    pub fn new() -> Self {
        CalculatedMovement {
            acc: Vec3::ZERO,
            vel: Vec3::ZERO,
            pos: Vec3::new(0.0, 2.0, 0.0),
        }
    }
}

pub fn path_finder(
    time: Res<Time>,
    arenito: Res<Arenito>,
    mut prev: ResMut<CalculatedMovement>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if arenito.state != ArenitoState::FORWARD {
        // not moving or movement not relevant,
        // velocity and acceleration are 0.
        prev.vel = Vec3::ZERO;
        prev.acc = Vec3::ZERO;
        return;
    }

    let accel = MPU6050::read_acc(&arenito);
    let gyro = MPU6050::read_rot(&arenito);

    // Previous movement values are stored in CalculatedMovement resource.
    // Initially, thay're set to 0. That is, Arenito initially is not moving.

    // Since the accelerometer only outputs ranges between 0 and 1024 a conversion
    // is needed to get the "real" acceleration direction vector.
    // This vector assumes a flat surface!
    // TODO: Direction vector for uneven surface.
    let acc = accel / 1024.0 * MPU6050::ACCELERATION_MAX;
    let acc = Vec3::new(gyro.y.cos(), 0.0, gyro.y.sin()) * acc.length();

    // get time `t` since last call (in seconds)
    let t = time.delta().as_millis() as f32 / 1000.0;

    // calculate current velocity
    // the real one won't need to calculate it, It'll have a velocimeter
    let mut vel = (acc * t) + prev.vel;
    if vel.length() > Arenito::MAX_VELOCITY {
        vel = vel.normalize() * Arenito::MAX_VELOCITY;
    }
    // calculate current position
    let d = (vel * t) + (0.5 * acc * t * t);
    let pos = prev.pos + d;
    // spawn wire
    Wire::spawn(
        prev.pos,
        pos,
        [1.0, 0.1, 1.0],
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    // update previous values
    prev.acc = acc;
    prev.vel = vel;
    prev.pos = pos;
}
