use bevy::{prelude::*, render::render_resource::*};

#[derive(Component, Copy, Clone)]
pub struct Wire {
    start: Vec3,
    end: Vec3,
}

impl Default for Wire {
    fn default() -> Self {
        Self {
            start: Vec3::ZERO,
            end: Vec3::new(1.0, 0.0, 0.0),
        }
    }
}

impl Wire {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Wire { start, end }
    }

    pub fn point(&mut self, end: Vec3, mesh: &mut Mesh) {
        self.end = end;
        let points = vec![self.start, self.end];
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
    }

    pub fn spawn(
        start: Vec3,
        end: Vec3,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let w = Wire::new(start, end);
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(w.into()),
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                ..default()
            },
            w,
        ));
    }
}

impl From<Wire> for Mesh {
    fn from(wire: Wire) -> Self {
        let Wire { start, end } = wire;

        let positions = vec![start, end];
        let normals = vec![[1.0, 1.0, 1.0]; 2];
        let uvs = vec![[1.0, 1.0]; 2];

        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

// TODO: Implement arrows
