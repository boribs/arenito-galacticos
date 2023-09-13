pub mod arenito;
pub mod sensor;
pub mod spatial_awareness;
pub mod static_shape;
pub mod wire;

use arenito::{ArenitoCamera, ArenitoPlugin};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::Viewport,
    window::WindowResized,
};
use spatial_awareness::*;

#[derive(Component)]
pub struct SceneCamera;
#[derive(Component)]
pub struct DataCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ArenitoPlugin { show_wires: false })
        .add_plugin(SpatialAwarenessPlugin)
        .add_startup_system(setup)
        .add_system(set_camera_viewports)
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
            transform: Transform::from_xyz(8.1, 4.0, 0.0)
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
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut right_camera: Query<&mut Camera, With<DataCamera>>,
    mut left_camera: Query<&mut Camera, (With<SceneCamera>, Without<DataCamera>)>,
    mut arenito_camera: Query<
        &mut Camera,
        (
            Without<SceneCamera>,
            Without<DataCamera>,
            With<ArenitoCamera>,
        ),
    >,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this
    // system for initial setup.
    // https://github.com/bevyengine/bevy/blob/main/examples/2d/2d_shapes.rs
    // https://github.com/bevyengine/bevy/blob/main/examples/3d/split_screen.rs
    for resize_event in resize_events.iter() {
        let window = windows.get(resize_event.window).unwrap();
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

        let (aw, ah) = (w / 5, h / 5);
        let mut arenito_camera = arenito_camera.single_mut();
        arenito_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(lw - aw - w / 37, h / 17),
            physical_size: UVec2::new(aw, ah),
            ..default()
        });
    }
}
