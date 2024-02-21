use crate::cans::*;
use crate::static_shape::Obstacle;
use bevy::{prelude::*, render::view::RenderLayers};
pub enum SceneName {
    Test,
    Basic,
    BasicCans,
    Obstacle,
    Deposit,
}

pub struct SceneLoaderPlugin {
    pub name: SceneName,
    pub display_can_collision_sphere: bool,
}

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CanManager::new());

        let f = match self.name {
            SceneName::Test => spawn_test_scene,
            SceneName::Basic => spawn_basic_plane_scene,
            SceneName::BasicCans => spawn_basic_scene_with_cans,
            SceneName::Obstacle => spawn_obstacle_scene,
            SceneName::Deposit => spawn_cans_with_deposit_scene,
        };

        app.add_systems(PreStartup, init_can_manager);
        app.add_systems(Startup, f);

        if self.display_can_collision_sphere {
            app.add_systems(Update, draw_can_collision);
        }
    }
}

fn spawn_plane(
    plane_size: f32,
    water_offset: f32,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let texture_handle = asset_server.load("textures/sand_01.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        reflectance: 0.01,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(plane_size).into()),
        material: material_handle,
        transform: Transform::from_xyz(0.0, 0.01, 0.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(plane_size + water_offset).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("0080FF").unwrap().into(),
            reflectance: 0.05,
            ..default()
        }),
        ..default()
    });
}

fn spawn_test_scene(
    _asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _can_manager: ResMut<CanManager>,
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

    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.01, 40.0, 0.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    },));
}

fn spawn_basic_plane_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut _can_manager: ResMut<CanManager>,
) {
    spawn_plane(
        15.0,
        2.0,
        &asset_server,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-0.4),
            ..default()
        },
        ..default()
    });

    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.01, 20.0, 0.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    },));
}

fn spawn_basic_scene_with_cans(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut can_manager: ResMut<CanManager>,
) {
    spawn_plane(
        15.0,
        2.0,
        &asset_server,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-0.4),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.01, 20.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
    ));

    let data = [
        //   x,   z,   ry
        (3.0, 3.0, 0.6),
        (5.0, 4.0, 0.9),
        (4.7, 0.4, 3.2),
        (3.5, -3.0, 2.1),
        (1.0, -5.0, 4.3),
        (-2.0, -1.3, 5.7),
        (-5.0, 4.3, 0.7),
        (-1.2, 2.0, 1.7),
        (-5.2, -4.0, 4.4),
    ];

    for d in data {
        let (x, z, ry) = d;

        can_manager.spawn(
            &mut commands,
            CanData {
                size: CanSize::Big,
                texture: CanTexture::Shiny,
            },
            Transform::from_xyz(x, 0.2, z).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.0,
                ry,
                1.56,
            )),
        );
    }
}

fn spawn_obstacle_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut _can_manager: ResMut<CanManager>,
) {
    spawn_plane(
        15.0,
        2.0,
        &asset_server,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-0.4),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.01, 20.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Obstacle::get_cube_mesh()),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
            transform: Transform::from_xyz(5.0, 0.0, 0.0),
            ..default()
        },
        Obstacle,
    ));
}

fn spawn_cans_with_deposit_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut can_manager: ResMut<CanManager>,
) {
    spawn_plane(
        15.0,
        2.0,
        &asset_server,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7500.0,
            ..default()
        },
        transform: Transform::from_xyz(3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7500.0,
            ..default()
        },
        transform: Transform::from_xyz(-3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.01, 20.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
    ));

    let data = [
        //   x,   z,   ry
        (3.0, 3.0, 0.6),
        (5.0, 4.0, 0.9),
        (4.7, 0.4, 3.2),
        (3.5, -3.0, 2.1),
        (1.0, -5.0, 4.3),
        (-2.0, -1.3, 5.7),
        (-5.0, 4.3, 0.7),
        (-1.2, 2.0, 1.7),
        (-5.2, -4.0, 4.4),
    ];

    for d in data {
        let (x, z, ry) = d;

        can_manager.spawn(
            &mut commands,
            CanData {
                size: CanSize::Big,
                texture: CanTexture::Shiny,
            },
            Transform::from_xyz(x, 0.2, z).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.0,
                ry,
                1.56,
            )),
        );
    }

    commands.spawn(PbrBundle {
        mesh: asset_server.load("models/deposit.obj"),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_xyz(-3.0, 0.0, 4.1),
        ..default()
    });
}
