pub mod arenito;
pub mod sensor;
pub mod spatial_awareness;
pub mod static_shape;
pub mod wire;

use arenito::ArenitoPlugin;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::Viewport,
    window::ExitCondition, winit::WinitSettings,
};
use sensor::AISimMem;
use shared_memory::*;
use spatial_awareness::*;

#[derive(Component)]
pub struct SceneCamera;
#[derive(Component)]
pub struct DataCamera;

fn main() {
    let flink = "shmem_arenito";
    let shmem: Shmem = match ShmemConf::new()
        .size(AISimMem::MIN_REQUIRED_MEMORY)
        .flink(flink)
        .create()
    {
        Ok(m) => {
            println!("created successfully");
            m
        }
        Err(ShmemError::LinkExists) => {
            println!("already exists. connecting.");
            ShmemConf::new().size(100).flink(flink).open().unwrap()
        }
        Err(_) => panic!("you did something very wrong."),
    };

    App::new()
        .add_plugins(DefaultPlugins.build().set(WindowPlugin {
            exit_condition: ExitCondition::OnPrimaryClosed,
            ..default()
        }))
        .insert_resource(WinitSettings {
            return_from_run: true,
            ..default()
        })
        .insert_resource(AISimMem::new(&shmem))
        .add_plugins((
            ArenitoPlugin {
                img_width: 512.0,
                img_height: 512.0,
            },
            SpatialAwarenessPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, set_camera_viewports)
        .run();
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

    // reference cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
        transform: Transform::from_xyz(4.0, 0.3, 0.0).with_scale(Vec3::splat(0.3)),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_xyz(3.1499052, 0.0, 0.3850749),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_xyz(1.4267371, 0.0, 0.0),
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

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.01, 40.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            camera: Camera {
                order: 1,
                ..default()
            },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        SceneCamera,
    ));

    commands.spawn((Camera2dBundle { ..default() }, DataCamera));
}

/// Dynamically resizes viewport widths according to window size.
/// This needs to be done every frame.
fn set_camera_viewports(
    mut window: Query<&Window, Without<arenito::ArenitoCamWindow>>,
    mut right_camera: Query<&mut Camera, With<DataCamera>>,
    mut left_camera: Query<&mut Camera, (With<SceneCamera>, Without<DataCamera>)>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this
    // system for initial setup.
    // https://github.com/bevyengine/bevy/blob/main/examples/2d/2d_shapes.rs
    // https://github.com/bevyengine/bevy/blob/main/examples/3d/split_screen.rs
    let window = window.single_mut();
    let (w, h) = (
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    );
    let lw = 3 * w / 5;
    let rw = w - lw;

    let mut left_camera = left_camera.single_mut();
    left_camera.viewport = Some(Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(lw, h),
        ..default()
    });

    let mut right_camera = right_camera.single_mut();
    right_camera.viewport = Some(Viewport {
        physical_position: UVec2::new(lw, 0),
        physical_size: UVec2::new(rw, h),
        ..default()
    });
}
