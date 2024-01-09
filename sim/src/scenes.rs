use bevy::prelude::*;

#[derive(Component)]
pub struct Can;

pub type SceneFunc =
    fn(Res<AssetServer>, Commands, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>);

#[derive(Resource)]
pub struct CanManager {
    material_handle: Option<Handle<StandardMaterial>>,
    mesh_handle: Option<Handle<Mesh>>,
}

impl CanManager {
    pub fn new() -> Self {
        CanManager {
            material_handle: None,
            mesh_handle: None,
        }
    }

    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
        transform: Transform,
    ) {
        if self.material_handle.is_none() {
            let texture_handle = asset_server.load("textures/black_01.png");
            self.material_handle = Some(materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle.clone()),
                reflectance: 0.4,
                ..default()
            }));
        }

        if self.mesh_handle.is_none() {
            let c = shape::Cylinder {
                radius: 0.15,
                height: 0.47,
                resolution: 15,
                segments: 1,
            };
            self.mesh_handle = Some(meshes.add(c.into()));
        }

        commands.spawn((
            PbrBundle {
                mesh: self.mesh_handle.clone().unwrap(),
                material: self.material_handle.clone().unwrap(),
                transform,
                ..default()
            },
            Can,
        ));
    }
}

pub type SceneFunc = fn(
    Res<AssetServer>,
    Commands,
    ResMut<Assets<Mesh>>,
    ResMut<Assets<StandardMaterial>>,
    ResMut<CanManager>,
);

pub enum SceneName {
    Test,
    Basic,
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
        };

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

    can_manager.spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        asset_server,
        Transform::from_xyz(4.0, 0.5, 0.0),
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
