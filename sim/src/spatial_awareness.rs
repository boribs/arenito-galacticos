use crate::arenito::*;
use crate::sensor::MPU6050;
use crate::wire::*;
use bevy::prelude::*;

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
    mut wirepath: ResMut<WirePath>,
    mut prev: ResMut<CalculatedMovement>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    segment_query: Query<(&mut Wire, &WirePathSegment, Entity, &Handle<Mesh>)>,
) {
    if arenito.state != ArenitoState::FORWARD {
        // not moving or movement not relevant,
        // velocity and acceleration are 0.
        prev.vel = Vec3::ZERO;
        prev.acc = Vec3::ZERO;
        return;
    }

    // Previously stopped, safe to assume new direction.
    if prev.vel == Vec3::ZERO {
        wirepath.append_segment(prev.pos, &mut commands, &mut meshes, &mut materials);
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

    // update current path segment
    wirepath.update_last(pos, &mut meshes, segment_query);

    // update previous values
    prev.acc = acc;
    prev.vel = vel;
    prev.pos = pos;
}
