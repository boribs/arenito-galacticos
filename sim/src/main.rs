use bevy::prelude::*;
use bevy_obj::*;

mod arenito;
use arenito::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .insert_resource(Arenito::new())
        .add_startup_system(setup)
        .add_startup_system(arenito_spawner)
        .run();
}

fn arenito_spawner(
    commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    arenito: Res<Arenito>
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
