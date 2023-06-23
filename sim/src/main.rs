pub mod arenito;
pub mod sensor;
pub mod wire;

use bevy::prelude::*;
use bevy_obj::*;

use arenito::*;
use wire::*;
use sensor::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .insert_resource(Arenito::new())
        .add_startup_system(setup)
        .add_startup_system(arenito_spawner)
        .add_system(arenito_mover)
        .add_system(sensor_reader)
        .run();
}

fn sensor_reader(arenito: Res<Arenito>) {
    let _accel_read = Sensor::read_acc(&arenito);
    // missing implementation of gyroscope
    // use gyroscope and accelerometer to determine position
}

fn arenito_mover(
    time: Res<Time>,
    commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut arenito: ResMut<Arenito>,
    body_part_query: Query<(&mut Transform, &BodyPart, Entity, With<BodyPart>)>,
) {
    if keyboard_input.pressed(KeyCode::W) {
        arenito.forward();
    } else if keyboard_input.pressed(KeyCode::A) {
        arenito.rotate(ArenitoState::LEFT);
    } else if keyboard_input.pressed(KeyCode::D) {
        arenito.rotate(ArenitoState::RIGHT);
    } else if keyboard_input.pressed(KeyCode::R) {
        arenito.reset(commands, asset_server, materials, &body_part_query);
    }

    arenito.update(time.delta().as_millis(), body_part_query);
    println!("{}", arenito.log());
}

fn arenito_spawner(
    commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    arenito: Res<Arenito>,
) {
    arenito.spawn(commands, materials, asset_server);
}

/// set up a simple 3D scene
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

    // let w = Wire::new(Vec3::new(0.0, 0.5, 0.0), Vec3::new(0., 2., 0.));
    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(w.into()),
    //         material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
    //         ..default()
    //     },
    //     w,
    // ));

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
        transform: Transform::from_xyz(-8.0, 5.0, 0.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
