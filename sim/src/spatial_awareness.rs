use crate::{arenito::SCALE_2D, arenito::*, sensor::MPU6050, wire::*};
use bevy::{prelude::*, sprite::Mesh2dHandle};

/// A plugin for Arenito's Spatial Awareness systems.
/// This plugin adds:
/// - Wire Path resource
/// - Calculated Movement resource
/// - Path Finder system
pub struct SpatialAwarenessPlugin;

#[derive(Component)]
pub struct A;

impl Plugin for SpatialAwarenessPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ArenitoPlugin>() {
            panic!("This plugin requires ArenitoPlugin!");
        }

        // resources
        app.insert_resource(CalculatedMovement::new());
        // startup systems
        app.add_startup_system(wirepath_init);
        // systems
        app.add_system(path_finder);
    }
}

/// Initializes a new path. This is required in order to start adding path segments.
fn wirepath_init(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    WirePath::spawn(
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::new(0.0, 0.0, 2.0),
        [0.0, 0.0, 1.0],
        A,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

/// This trait aims to unify the calculation of a direction vector from
/// the output of MPU6050's gyroscope.
/// Tailored specifically for this simulator's application it's assumed
/// that the X+ axis points (initially) forwards and it controls the roll
/// of Arenito.
///
/// To determine the direction only the yaw and the pitch are considered.
/// That means that the y (yaw) and z (pitch) components of the gyro values
/// are used.
pub trait FromGyro {
    fn from_gyro(gyro: &Vec3) -> Vec3;
}

impl FromGyro for Vec3 {
    /// Creates a unit vector from a rotation vector.
    fn from_gyro(gyro: &Vec3) -> Vec3 {
        Vec3::new(gyro.y.cos(), gyro.z.sin(), gyro.y.sin())
    }
}

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
    /// Creates a new CalculatedMovement instance set to the default values.
    pub fn new() -> Self {
        CalculatedMovement {
            acc: Vec3::ZERO,
            vel: Vec3::ZERO,
            pos: Vec3::new(0.0, 2.0, 0.0),
        }
    }
}

/// Routine to determine the path Arenito has taken.
/// This also creates wires that represent tis path.
///
/// This path prediction works by reading the outputs from the
/// MPU6050 sensor and calculating how much the robot has moved
/// (and where) based on previous movement values, remembered by
/// the CalculatedMovement resource.
pub fn path_finder(
    time: Res<Time>,
    arenito: Res<Arenito>,
    mut wirepath: Query<(&mut WirePath, &Mesh2dHandle, With<A>)>,
    mut prev: ResMut<CalculatedMovement>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if arenito.state != ArenitoState::FORWARD {
        // not moving or movement not relevant,
        // velocity and acceleration are 0.
        prev.vel = Vec3::ZERO;
        prev.acc = Vec3::ZERO;
        return;
    }

    let (mut wirepath, handle, _) = wirepath.single_mut();

    // Previously stopped, safe to assume new direction.
    if prev.vel == Vec3::ZERO {
        wirepath.append_segment(prev.pos);
    }

    let accel = MPU6050::read_acc(&arenito);
    let gyro = MPU6050::read_rot(&arenito);

    // Previous movement values are stored in CalculatedMovement resource.
    // Initially, thay're set to 0. That is, Arenito initially is not moving.

    // Since the accelerometer only outputs ranges between 0 and 1024 a conversion
    // is needed to get the "real" acceleration direction vector.
    // This vector assumes a flat surface!
    let acc = accel / 1024.0 * MPU6050::ACCELERATION_MAX;
    let acc = Vec3::from_gyro(&gyro) * acc.length();

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
    wirepath.move_last(pos.to_2d() * SCALE_2D);
    // update wirepath mesh
    wirepath.update(meshes.get_mut(&handle.0).unwrap());

    // update previous values
    prev.acc = acc;
    prev.vel = vel;
    prev.pos = pos;
}
