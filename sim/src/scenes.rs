use crate::cans::*;
use crate::static_shape::Obstacle;
use bevy::{prelude::*, render::view::RenderLayers, scene};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub struct SceneLoaderPlugin {
    pub scene_data: SceneData,
    pub display_can_collision_sphere: bool,
}

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CanManager::new())
            .insert_resource(self.scene_data.clone())
            .add_plugins(PanOrbitCameraPlugin);

        app.add_systems(PreStartup, init_can_manager);
        app.add_systems(Startup, generate_scene);

        if self.display_can_collision_sphere {
            app.add_systems(Update, draw_can_collision);
        }
    }
}

/// A material abstraction, this is used to simplify specifying whether
/// a material shoud be color or texture based.
#[derive(Copy, Clone)]
pub enum TextureOrColor {
    Texture(&'static str),
    Color(Color),
}

/// Stores everything contained initially on the scene.
#[derive(Resource, Clone)]
pub struct SceneData {
    cam_transform: Transform,
    sand: PlaneData,
    water: PlaneData,
    can_positions: Vec<(f32, f32, f32)>,
    deposit_position: Vec3,
}

impl SceneData {
    /// Sets the size of the sand and water planes.
    pub fn base_size(mut self, width: f32, length: f32, water_offset: f32) -> Self {
        self.sand.length = length;
        self.sand.width = width;
        self.water.length = length + water_offset;
        self.water.width = width + water_offset;
        self
    }

    /// Sets can positions.
    pub fn cans(mut self, can_positions: Vec<(f32, f32, f32)>) -> Self {
        self.can_positions = can_positions;
        self
    }
}

impl Default for SceneData {
    fn default() -> Self {
        SceneData {
            cam_transform: Transform::from_xyz(0.0, 20.0, 0.01).looking_at(Vec3::ZERO, Vec3::Y),
            sand: PlaneData::sand(15.0, 25.0, 0.01),
            water: PlaneData::water(17.0, 27.0, 0.01),
            can_positions: vec![
                (3.0, 3.0, 0.6),
                (5.0, 4.0, 0.9),
                (4.7, 0.4, 3.2),
                (3.5, -3.0, 2.1),
                (1.0, -5.0, 4.3),
                (-2.0, -1.3, 5.7),
                (-7.0, 4.3, 0.7),
                (-1.2, 2.0, 1.7),
                (-5.2, -4.0, 4.4),
                (-11.4, 5.7, 0.4),
                (-8.1, 0.7, 5.1),
                (9.1, -4.7, 0.0),
                (11.1, 1.2, 0.0),
                (10.4, 4.7, 0.0),
            ],
            deposit_position: Vec3::new(-3.0, 0.0, 4.1),
        }
    }
}

/// Stores plane data. Used for base planes (sand and water).
#[derive(Copy, Clone)]
pub struct PlaneData {
    base: TextureOrColor,
    width: f32,
    length: f32,
    reflectance: f32,
}

impl PlaneData {
    /// Plane data for sand.
    pub fn sand(width: f32, length: f32, reflectance: f32) -> Self {
        PlaneData {
            base: TextureOrColor::Texture("textures/sand_01.png"),
            width,
            length,
            reflectance,
        }
    }

    /// Plane data for sand.
    pub fn water(width: f32, length: f32, reflectance: f32) -> Self {
        PlaneData {
            base: TextureOrColor::Color(Color::hex("0080FF").unwrap()),
            width,
            length,
            reflectance,
        }
    }

    /// Returns rendering material from self.
    pub fn get_material(&self, asset_server: &Res<AssetServer>) -> StandardMaterial {
        match self.base {
            TextureOrColor::Color(c) => StandardMaterial {
                base_color: c,
                reflectance: self.reflectance,
                ..Default::default()
            },
            TextureOrColor::Texture(t) => {
                let texture_handle = asset_server.load(t);
                StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    reflectance: self.reflectance,
                    ..default()
                }
            }
        }
    }
}

fn generate_scene(
    scene_data: Res<SceneData>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut can_manager: ResMut<CanManager>,
) {
    // spawn base
    let water_material = scene_data.water.get_material(&asset_server);
    let water_scale = Vec3::new(scene_data.water.length, 1.0, scene_data.water.width);
    let sand_material = scene_data.sand.get_material(&asset_server);
    let sand_scale = Vec3::new(scene_data.sand.length, 1.0, scene_data.sand.width);

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(1.0).into()),
        material: materials.add( water_material),
        transform: Transform::from_scale(water_scale),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(1.0).into()),
        material: materials.add(sand_material),
        transform: Transform::from_xyz(0.0, 0.01, 0.0).with_scale(sand_scale),
        ..default()
    });

    // spawn lights
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7500.0,
            ..default()
        },
        transform: Transform::from_xyz(3.0, 2.0,-1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7500.0,
            ..default()
        },
        transform: Transform::from_xyz(-3.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // spawn camera
    commands.spawn((
        Camera3dBundle {
            transform: scene_data.cam_transform,
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
        PanOrbitCamera::default(),
    ));

    // spawn cans
    for d in scene_data.can_positions.iter() {
        let (x, z, ry) = d;

        can_manager.spawn(
            &mut commands,
            CanData {
                size: CanSize::Big,
                texture: CanTexture::Shiny,
            },
            Transform::from_xyz(*x, 0.2, *z).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.0,
                *ry,
                1.56,
            )),
        );
    }

    // spawn deposit
    commands.spawn(PbrBundle {
        mesh: asset_server.load("models/deposit.obj"),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_translation(scene_data.deposit_position),
        ..default()
    });

}

// fn spawn_plane(
//     plane_size: f32,
//     water_offset: f32,
//     scale: Vec3,
//     asset_server: &Res<AssetServer>,
//     commands: &mut Commands,
//     meshes: &mut ResMut<Assets<Mesh>>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
// ) {
//     let texture_handle = asset_server.load("textures/sand_01.png");
//     let material_handle = materials.add(StandardMaterial {
//         base_color_texture: Some(texture_handle.clone()),
//         reflectance: 0.01,
//         ..default()
//     });

//     commands.spawn(PbrBundle {
//         mesh: meshes.add(shape::Plane::from_size(plane_size).into()),
//         material: material_handle,
//         transform: Transform::from_xyz(0.0, 0.01, 0.0).with_scale(scale),
//         ..default()
//     });

//     commands.spawn(PbrBundle {
//         mesh: meshes.add(shape::Plane::from_size(plane_size + water_offset).into()),
//         material: materials.add(StandardMaterial {
//             base_color: Color::hex("0080FF").unwrap().into(),
//             reflectance: 0.05,
//             ..default()
//         }),
//         transform: Transform::from_scale(scale),
//         ..default()
//     });
// }

// fn spawn_chair(
//     frame_color: Color,
//     cloth_color: Color,
//     transform: Transform,
//     commands: &mut Commands,
//     asset_server: &Res<AssetServer>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
// ) {
//     commands.spawn((
//         PbrBundle {
//             mesh: asset_server.load("models/silla-marco.obj"),
//             material: materials.add(StandardMaterial {
//                 base_color: frame_color,
//                 ..default()
//             }),
//             transform,
//             ..default()
//         },
//         Obstacle,
//     ));
//     commands.spawn((
//         PbrBundle {
//             mesh: asset_server.load("models/silla-tela.obj"),
//             material: materials.add(StandardMaterial {
//                 base_color: cloth_color,
//                 ..default()
//             }),
//             transform,
//             ..default()
//         },
//         Obstacle,
//     ));
// }

// fn spawn_test_scene(
//     _asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     _can_manager: ResMut<CanManager>,
// ) {
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(shape::Plane::from_size(10.0).into()),
//         material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
//         ..default()
//     });

//     // reference cube
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//         material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
//         transform: Transform::from_xyz(4.0, 0.3, 0.0).with_scale(Vec3::splat(0.3)),
//         ..default()
//     });

//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
//         material: materials.add(Color::WHITE.into()),
//         transform: Transform::from_xyz(3.1499052, 0.0, 0.3850749),
//         ..default()
//     });

//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
//         material: materials.add(Color::RED.into()),
//         transform: Transform::from_xyz(1.4267371, 0.0, 0.0),
//         ..default()
//     });

//     commands.spawn(PointLightBundle {
//         point_light: PointLight {
//             intensity: 1500.0,
//             shadows_enabled: false,
//             ..default()
//         },
//         transform: Transform::from_xyz(4.0, 8.0, 4.0),
//         ..default()
//     });

//     commands.spawn((Camera3dBundle {
//         transform: Transform::from_xyz(0.01, 40.0, 0.0)
//             .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
//         ..default()
//     },));
// }

// fn spawn_basic_plane_scene(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut _can_manager: ResMut<CanManager>,
// ) {
//     spawn_plane(
//         15.0,
//         2.0,
//         Vec3::ONE,
//         &asset_server,
//         &mut commands,
//         &mut meshes,
//         &mut materials,
//     );

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 15000.0,
//             ..default()
//         },
//         transform: Transform {
//             translation: Vec3::new(0.0, 2.0, 0.0),
//             rotation: Quat::from_rotation_x(-0.4),
//             ..default()
//         },
//         ..default()
//     });

//     commands.spawn((Camera3dBundle {
//         transform: Transform::from_xyz(0.01, 20.0, 0.0)
//             .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
//         ..default()
//     },));
// }

// fn spawn_basic_scene_with_cans(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut can_manager: ResMut<CanManager>,
// ) {
//     spawn_plane(
//         15.0,
//         2.0,
//         Vec3::ONE,
//         &asset_server,
//         &mut commands,
//         &mut meshes,
//         &mut materials,
//     );

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 15000.0,
//             ..default()
//         },
//         transform: Transform {
//             translation: Vec3::new(0.0, 2.0, 0.0),
//             rotation: Quat::from_rotation_x(-0.4),
//             ..default()
//         },
//         ..default()
//     });

//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_xyz(0.01, 20.0, 0.0)
//                 .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
//             ..default()
//         },
//         RenderLayers::from_layers(&[0, 1]),
//     ));

//     let data = [
//         //   x,   z,   ry
//         (3.0, 3.0, 0.6),
//         (5.0, 4.0, 0.9),
//         (4.7, 0.4, 3.2),
//         (3.5, -3.0, 2.1),
//         (1.0, -5.0, 4.3),
//         (-2.0, -1.3, 5.7),
//         (-5.0, 4.3, 0.7),
//         (-1.2, 2.0, 1.7),
//         (-5.2, -4.0, 4.4),
//     ];

//     for d in data {
//         let (x, z, ry) = d;

//         can_manager.spawn(
//             &mut commands,
//             CanData {
//                 size: CanSize::Big,
//                 texture: CanTexture::Shiny,
//             },
//             Transform::from_xyz(x, 0.2, z).with_rotation(Quat::from_euler(
//                 EulerRot::XYZ,
//                 0.0,
//                 ry,
//                 1.56,
//             )),
//         );
//     }
// }

// fn spawn_obstacle_scene(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut _can_manager: ResMut<CanManager>,
// ) {
//     spawn_plane(
//         15.0,
//         2.0,
//         Vec3::ONE,
//         &asset_server,
//         &mut commands,
//         &mut meshes,
//         &mut materials,
//     );

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 15000.0,
//             ..default()
//         },
//         transform: Transform {
//             translation: Vec3::new(0.0, 2.0, 0.0),
//             rotation: Quat::from_rotation_x(-0.4),
//             ..default()
//         },
//         ..default()
//     });

//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_xyz(4.01, 2.0, 4.0)
//                 .looking_at(Vec3::new(0.0, 0.0, 5.0), Vec3::Y),
//             ..default()
//         },
//         RenderLayers::from_layers(&[0, 1]),
//         PanOrbitCamera::default(),
//     ));

//     spawn_chair(
//         Color::DARK_GRAY,
//         Color::GREEN,
//         Transform::from_xyz(0.0, 0.0, 5.0).with_rotation(Quat::from_euler(
//             EulerRot::XYZ,
//             0.0,
//             0.5,
//             0.0,
//         )),
//         &mut commands,
//         &asset_server,
//         &mut materials,
//     );
// }

// fn spawn_cans_with_deposit_scene(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut can_manager: ResMut<CanManager>,
// ) {
//     spawn_plane(
//         15.0,
//         2.0,
//         Vec3::ONE,
//         &asset_server,
//         &mut commands,
//         &mut meshes,
//         &mut materials,
//     );

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 7500.0,
//             ..default()
//         },
//         transform: Transform::from_xyz(3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 7500.0,
//             ..default()
//         },
//         transform: Transform::from_xyz(-3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });

//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_xyz(0.01, 20.0, 0.0)
//                 .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
//             ..default()
//         },
//         RenderLayers::from_layers(&[0, 1]),
//         PanOrbitCamera::default(),
//     ));

//     let data = [
//         //   x,   z,   ry
//         (3.0, 3.0, 0.6),
//         (5.0, 4.0, 0.9),
//         (4.7, 0.4, 3.2),
//         (3.5, -3.0, 2.1),
//         (1.0, -5.0, 4.3),
//         (-2.0, -1.3, 5.7),
//         (-7.0, 4.3, 0.7),
//         (-1.2, 2.0, 1.7),
//         (-5.2, -4.0, 4.4),
//     ];

//     for d in data {
//         let (x, z, ry) = d;

//         can_manager.spawn(
//             &mut commands,
//             CanData {
//                 size: CanSize::Big,
//                 texture: CanTexture::Shiny,
//             },
//             Transform::from_xyz(x, 0.2, z).with_rotation(Quat::from_euler(
//                 EulerRot::XYZ,
//                 0.0,
//                 ry,
//                 1.56,
//             )),
//         );
//     }

//     commands.spawn(PbrBundle {
//         mesh: asset_server.load("models/deposit.obj"),
//         material: materials.add(Color::RED.into()),
//         transform: Transform::from_xyz(-3.0, 0.0, 4.1),
//         ..default()
//     });
// }

// fn spawn_full_scene_1(
//     asset_server: Res<AssetServer>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut can_manager: ResMut<CanManager>,
// ) {
//     spawn_plane(
//         15.0,
//         2.0,
//         Vec3::new(5.0 / 3.0, 1.0, 1.0),
//         &asset_server,
//         &mut commands,
//         &mut meshes,
//         &mut materials,
//     );

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 7500.0,
//             ..default()
//         },
//         transform: Transform::from_xyz(3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             illuminance: 7500.0,
//             ..default()
//         },
//         transform: Transform::from_xyz(-3.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });

//     commands.spawn((
//         Camera3dBundle {
//             transform: Transform::from_xyz(0.01, 20.0, 0.0)
//                 .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
//             ..default()
//         },
//         RenderLayers::from_layers(&[0, 1]),
//         PanOrbitCamera::default(),
//     ));

//     let data = [
//         //   x,   z,   ry
//         (3.0, 3.0, 0.6),
//         (5.0, 4.0, 0.9),
//         (4.7, 0.4, 3.2),
//         (3.5, -3.0, 2.1),
//         (1.0, -5.0, 4.3),
//         (-2.0, -1.3, 5.7),
//         (-7.0, 4.3, 0.7),
//         (-1.2, 2.0, 1.7),
//         (-5.2, -4.0, 4.4),
//         (-11.4, 5.7, 0.4),
//         (-8.1, 0.7, 5.1),
//         (9.1, -4.7, 0.0),
//         (11.1, 1.2, 0.0),
//         (10.4, 4.7, 0.0),
//     ];

//     for d in data {
//         let (x, z, ry) = d;

//         can_manager.spawn(
//             &mut commands,
//             CanData {
//                 size: CanSize::Big,
//                 texture: CanTexture::Shiny,
//             },
//             Transform::from_xyz(x, 0.2, z).with_rotation(Quat::from_euler(
//                 EulerRot::XYZ,
//                 0.0,
//                 ry,
//                 1.56,
//             )),
//         );
//     }

//     commands.spawn(PbrBundle {
//         mesh: asset_server.load("models/deposit.obj"),
//         material: materials.add(Color::RED.into()),
//         transform: Transform::from_xyz(-3.0, 0.0, 4.1),
//         ..default()
//     });

//     spawn_chair(
//         Color::MIDNIGHT_BLUE,
//         Color::GREEN,
//         Transform::from_xyz(-8.0, 0.0, -5.0).with_rotation(Quat::from_euler(
//             EulerRot::XYZ,
//             0.0,
//             0.5,
//             0.0,
//         )),
//         &mut commands,
//         &asset_server,
//         &mut materials,
//     );
// }
