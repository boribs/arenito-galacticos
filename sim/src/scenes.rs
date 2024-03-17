use crate::cans::*;
use crate::collision::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub struct SceneLoaderPlugin {
    pub scene_data: SceneData,
    pub display_can_collision_sphere: bool,
    pub draw_obstacle_collision_mesh: bool,
}

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CanManager::new())
            .insert_resource(self.scene_data.clone())
            .add_plugins(PanOrbitCameraPlugin);

        app.add_systems(PreStartup, init_can_manager);
        app.add_systems(PreStartup, generate_scene);

        if self.display_can_collision_sphere {
            app.add_systems(Update, draw_can_collision_sphere);
        }
        if self.draw_obstacle_collision_mesh {
            app.add_systems(PreUpdate, compute_hulls);
            app.add_systems(Update, draw_obstacle_collision_mesh);
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

impl TextureOrColor {
    /// Returns rendering material from self.
    pub fn get_material(
        &self,
        reflectance: f32,
        asset_server: &Res<AssetServer>,
    ) -> StandardMaterial {
        match self {
            TextureOrColor::Color(c) => StandardMaterial {
                base_color: *c,
                reflectance,
                ..Default::default()
            },
            TextureOrColor::Texture(t) => {
                let texture_handle = asset_server.load(*t);
                StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    reflectance,
                    ..default()
                }
            }
        }
    }
}

/// Stores everything contained initially on the scene.
#[derive(Resource, Clone)]
pub struct SceneData {
    cam_transform: Transform,
    sand: PlaneData,
    water: PlaneData,
    can_positions: Vec<(f32, f32, f32)>,
    deposit_position: Vec3,
    obstacles: Vec<ObstacleData>,
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
            obstacles: vec![ObstacleData {
                models: vec![
                    (
                        "models/silla-marco.obj",
                        TextureOrColor::Color(Color::DARK_GRAY),
                    ),
                    ("models/silla-tela.obj", TextureOrColor::Color(Color::GREEN)),
                ],
                transform: Transform::from_xyz(-8.0, 0.0, -5.0).with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0.0,
                    0.5,
                    0.0,
                )),
            }],
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
        self.base.get_material(self.reflectance, &asset_server)
    }
}

#[derive(Clone)]
pub struct ObstacleData {
    models: Vec<(&'static str, TextureOrColor)>,
    transform: Transform,
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
        material: materials.add(water_material),
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
        transform: Transform::from_xyz(3.0, 2.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    // spawn obstacles
    for obstacle in scene_data.obstacles.iter() {
        for model in obstacle.models.iter() {
            commands.spawn((
                PbrBundle {
                    mesh: asset_server.load(model.0),
                    material: materials.add(model.1.get_material(0.3, &asset_server)),
                    transform: obstacle.transform.clone(),
                    ..default()
                },
                Obstacle,
            ));
        }
    }
}

pub fn draw_can_collision_sphere(mut gizmos: Gizmos, cans: Query<(&CanData, &Transform)>) {
    for (can, transform) in cans.iter() {
        can.draw_sphere(transform, Color::WHITE, &mut gizmos);
    }
}

pub fn draw_obstacle_collision_mesh(
    mut gizmos: Gizmos,
    meshes: Res<Assets<Mesh>>,
    obstacles: Query<(&Obstacle, &Handle<Mesh>, &Transform)>,
) {
    for obstacle in obstacles.iter() {
        let mesh = meshes.get(obstacle.1).unwrap();
        let hull = obstacle.0.compute_hull(mesh, obstacle.2);

        for triangle in hull {
            gizmos.line(triangle.a, triangle.b, Color::WHITE);
            gizmos.line(triangle.b, triangle.c, Color::WHITE);
            gizmos.line(triangle.a, triangle.c, Color::WHITE);
        }
    }
}
