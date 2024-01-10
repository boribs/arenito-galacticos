use bevy::prelude::*;

#[derive(Component)]
pub struct Can;

pub struct CanData {
    pos: Vec3,
    rot: Quat,
    size: CanSize,
    texture: CanTexture,
}

pub enum CanSize {
    Big,
    Small,
}

pub enum CanTexture {
    Shiny,
    Dirty,
}

#[derive(Resource)]
pub struct CanManager {
    dirty_material_handle: Option<Handle<StandardMaterial>>,
    shiny_material_handle: Option<Handle<StandardMaterial>>,
    big_mesh_handle: Option<Handle<Mesh>>,
    small_mesh_handle: Option<Handle<Mesh>>,
}

impl CanManager {
    pub fn new() -> Self {
        CanManager {
            dirty_material_handle: None,
            shiny_material_handle: None,
            big_mesh_handle: None,
            small_mesh_handle: None,
        }
    }

    fn load_textures(
        &mut self,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        self.dirty_material_handle = Some(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/black_01.png")),
            reflectance: 0.3,
            ..default()
        }));

        self.shiny_material_handle = Some(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/black_02.png")),
            reflectance: 0.34,
            ..default()
        }));
    }

    fn load_meshes(&mut self, mut meshes: ResMut<Assets<Mesh>>) {
        self.big_mesh_handle = Some(
            meshes.add(
                shape::Cylinder {
                    radius: 0.15,
                    height: 0.47,
                    resolution: 15,
                    segments: 1,
                }
                .into(),
            ),
        );
        self.small_mesh_handle = Some(
            meshes.add(
                shape::Cylinder {
                    radius: 0.15,
                    height: 0.37,
                    resolution: 15,
                    segments: 1,
                }
                .into(),
            ),
        );
    }

    pub fn spawn(&mut self, commands: &mut Commands, can_data: CanData) {
        let mesh = match can_data.size {
            CanSize::Big => self.big_mesh_handle.clone().unwrap(),
            CanSize::Small => self.small_mesh_handle.clone().unwrap(),
        };

        let material = match can_data.texture {
            CanTexture::Shiny => self.shiny_material_handle.clone().unwrap(),
            CanTexture::Dirty => self.dirty_material_handle.clone().unwrap(),
        };

        commands.spawn((
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_xyz(can_data.pos.x, can_data.pos.y, can_data.pos.z)
                    .with_rotation(can_data.rot),
                ..default()
            },
            Can,
        ));
    }
}

fn init_can_manager(
    mut can_manager: ResMut<CanManager>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    can_manager.load_meshes(meshes);
    can_manager.load_textures(materials, asset_server);
}

pub enum SceneName {
    Test,
    Basic,
    BasicCans,
}

pub struct SceneLoaderPlugin {
    pub name: SceneName,
}

impl Plugin for SceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CanManager::new());

        let f = match self.name {
            SceneName::Test => spawn_test_scene,
            SceneName::Basic => spawn_basic_plane_scene,
            SceneName::BasicCans => spawn_basic_scene_with_cans,
        };

        app.add_systems(PreStartup, init_can_manager);
        app.add_systems(Startup, f);
    }
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
    let texture_handle = asset_server.load("textures/sand_01.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        reflectance: 0.01,
        ..default()
    });

    let plane_size = 15.0;
    let water_offset = 2.0;

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
    let texture_handle = asset_server.load("textures/sand_01.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        reflectance: 0.01,
        ..default()
    });

    let plane_size = 15.0;
    let water_offset = 2.0;

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
                pos: Vec3::new(x, 0.2, z),
                rot: Quat::from_euler(EulerRot::XYZ, 0.0, ry, 1.56),
                size: CanSize::Big,
                texture: CanTexture::Shiny,
            },
        );
    }
}
