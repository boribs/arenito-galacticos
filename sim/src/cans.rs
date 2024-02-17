use crate::collision::DistanceCollision;
use bevy::prelude::*;

pub enum CanSize {
    Big,
    Small,
}

pub enum CanTexture {
    Shiny,
    Dirty,
}

#[derive(Component)]
pub struct CanData {
    pub size: CanSize,
    pub texture: CanTexture,
}

impl Default for CanData {
    fn default() -> Self {
        CanData {
            size: CanSize::Big,
            texture: CanTexture::Shiny,
        }
    }
}

impl DistanceCollision for CanData {
    fn get_radius(&self) -> f32 {
        0.3
    }
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

    pub fn spawn(&mut self, commands: &mut Commands, can_data: CanData, can_transform: Transform) {
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
                transform: can_transform,
                ..default()
            },
            can_data,
        ));
    }
}

pub fn init_can_manager(
    mut can_manager: ResMut<CanManager>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    can_manager.load_meshes(meshes);
    can_manager.load_textures(materials, asset_server);
}

pub fn draw_can_collision(mut gizmos: Gizmos, cans: Query<(&CanData, &Transform)>) {
    for (can, transform) in cans.iter() {
        can.draw_sphere(transform, Color::WHITE, &mut gizmos);
    }
}
