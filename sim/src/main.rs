pub mod arenito;
pub mod sensor;
pub mod wire;

use bevy::prelude::*;
use bevy_obj::*;

use arenito::*;
use sensor::*;
use wire::*;

#[derive(Component)]
enum WireComponent {
    VELOCITY,
    ACCELERATOIN,
    ROTATION,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .insert_resource(Arenito::new())
        .insert_resource(CalculatedMovement::new())
        .add_startup_system(setup)
        .add_startup_system(arenito_spawner)
        .add_system(arenito_mover)
        .add_system(wire_mover)
        .add_system(sensor_reader)
        .run();
}

fn wire_mover(
    arenito: ResMut<Arenito>,
    mut wire: Query<(&mut Wire, &Handle<Mesh>, &WireComponent)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut vel: Option<(Mut<'_, Wire>, &Handle<Mesh>, &WireComponent)> = None;
    let mut acc: Option<(Mut<'_, Wire>, &Handle<Mesh>, &WireComponent)> = None;
    let mut rot: Option<(Mut<'_, Wire>, &Handle<Mesh>, &WireComponent)> = None;

    for w in &mut wire {
        match w.2 {
            WireComponent::ACCELERATOIN => acc = Some(w),
            WireComponent::VELOCITY => vel = Some(w),
            WireComponent::ROTATION => rot = Some(w),
        }
    }

    let mut vel = vel.unwrap();
    let mut acc = acc.unwrap();
    let mut rot = rot.unwrap();

    let rup = Vec3::new(0.0, 1.6, 0.0);
    let vup = Vec3::new(0.0, 1.7, 0.0);
    let aup = Vec3::new(0.0, 1.8, 0.0);

    vel.0.set_start(arenito.center + vup);
    vel.0.set_end(arenito.center + arenito.vel + vup);
    vel.0.update(meshes.get_mut(vel.1).unwrap());

    acc.0.set_start(arenito.center + aup);
    acc.0.set_end(arenito.center + arenito.acc + aup);
    acc.0.update(meshes.get_mut(acc.1).unwrap());

    let r = MPU6050::read_rot(&arenito);
    // let r = arenito.rot;
    let rvec = Vec3::new(r.y.cos(), 0.0, r.y.sin());
    rot.0.set_start(arenito.center + rup);
    rot.0.set_end(arenito.center + rvec + rup);
    rot.0.update(meshes.get_mut(rot.1).unwrap());
}

fn sensor_reader(
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

fn arenito_mover(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arenito: ResMut<Arenito>,
    body_part_query: Query<(&mut Transform, &BodyPart, Entity)>,
) {
    if keyboard_input.pressed(KeyCode::W) {
        arenito.forward();
    } else if keyboard_input.pressed(KeyCode::A) {
        arenito.rotate(ArenitoState::LEFT);
    } else if keyboard_input.pressed(KeyCode::D) {
        arenito.rotate(ArenitoState::RIGHT);
    } else if keyboard_input.pressed(KeyCode::R) {
        arenito.reset(
            &mut commands,
            &asset_server,
            &mut materials,
            &body_part_query,
        );
    }

    arenito.update(time.delta().as_millis(), body_part_query);
    // println!("{}", arenito.log());
}

fn arenito_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    arenito: Res<Arenito>,
) {
    arenito.spawn(&mut commands, &mut materials, &asset_server);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(10.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    Wire::spawn_unique(
        Vec3::ZERO,
        Vec3::ZERO,
        [1.0, 1.0, 0.0],
        WireComponent::VELOCITY,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    Wire::spawn_unique(
        Vec3::ZERO,
        Vec3::ZERO,
        [1.0, 0.0, 0.0],
        WireComponent::ACCELERATOIN,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    Wire::spawn_unique(
        Vec3::ZERO,
        Vec3::ZERO,
        [0.0, 0.0, 1.0],
        WireComponent::ROTATION,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.1, 12.0, 0.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
