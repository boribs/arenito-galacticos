use bevy::prelude::*;
use bevy_obj::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: asset_server.load("arenito.obj"),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: asset_server.load("rueda.obj"),
        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
        transform: Transform::from_xyz(0.5, 0.5, 0.85),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: asset_server.load("rueda.obj"),
        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
        transform: Transform::from_xyz(-0.5, 0.5, 0.85),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: asset_server.load("rueda.obj"),
        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
        transform: Transform::from_xyz(0.5, 0.5, -0.85),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: asset_server.load("rueda.obj"),
        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
        transform: Transform::from_xyz(-0.5, 0.5, -0.85),
        ..default()
    });
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
        transform: Transform::from_xyz(2.0, 2.5, 7.0)
                        .looking_at(Vec3::new(0.0, 0.0, -7.0), Vec3::Y),
        ..default()
    });
}
