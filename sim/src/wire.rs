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
    /// Creates a new Wire component.
    /// This is supposed to be passed on spawn and then retrieved
    /// with a Query.
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Wire { start, end }
    }

    /// Sets the start coordinate of the wire.
    /// Does not update on the mesh!
    pub fn set_start(&mut self, start: Vec3) {
        self.start = start;
    }

    /// Sets the end coordinate of the wire.
    /// Does not update on the mesh!
    pub fn set_end(&mut self, end: Vec3) {
        self.end = end;
    }

    /// Updates the mesh of the wire.
    /// Call this after setting final start and end points.
    pub fn update(&self, mesh: &mut Mesh) {
        // https://stackoverflow.com/questions/72961896/how-do-i-modify-a-mesh-after-it-has-been-created-in-bevy-rust
        // i keep forgetting how to get the mesh...
        let points = vec![self.start, self.end];
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
    }

    /// Spawns a Wire on a given position.
    pub fn spawn(
        start: Vec3,
        end: Vec3,
        color: [f32; 3],
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let w = Wire::new(start, end);
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(w.into()),
                material: materials.add(Color::from(color).into()),
                ..default()
            },
            w,
        ));
    }

    /// Spawns a Wire on a given position with another component.
    pub fn spawn_unique<C>(
        start: Vec3,
        end: Vec3,
        color: [f32; 3],
        component: C,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) where
        C: Component,
    {
        let w = Wire::new(start, end);
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(w.into()),
                material: materials.add(Color::from(color).into()),
                ..default()
            },
            w,
            component,
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
