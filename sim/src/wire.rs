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

    /// Spawns a Wire on a given position with another component.
    pub fn spawn3d<C>(
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


// TODO: Don't use wires for WirePath, instead modify it's mesh.

/// This struct is to (visually) describe paths!
#[derive(Component)]
pub struct WirePath {
    pub color: [f32; 3],
    pub segments: Vec<Vec3>,
}

impl WirePath {
    /// Creates a new WirePath instance. This will be used to control the path.
    /// A path is created with new_segment
    pub fn new(color: [f32; 3]) -> Self {
        WirePath {
            color,
            segments: Vec::new(),
        }
    }

    /// Initializes a path. Creates a segment from `start` to `end`.
    /// This method must be called once at the beginning of the path.
    /// Subsecuent calls with an already initialized path will result in a panic.
    pub fn init_path(
        &mut self,
        start: Vec3,
        end: Vec3,
    ) {
        if !self.segments.is_empty() {
            panic!("This method must be called only once!");
        }

        self.segments.push(start);
        self.segments.push(end);
    }

    /// Adds a new end point to the wire.
    pub fn append_segment(
        &mut self,
        end: Vec3,
    ) {
        if self.segments.is_empty() {
            panic!("Must initialize a path before adding segments!");
        }

        self.segments.push(end);
    }

    /// Deletes the last path segment.
    pub fn delete_last(
        &mut self,
    ) {
        if self.segments.len() < 3 { // check this!
            return;
        }

        self.segments.pop();
    }

    /// Updates the end position of the last path segment.
    pub fn move_last(
        &mut self,
        end: Vec3,
    ) {
        if self.segments.is_empty() {
            panic!("No segments in path!");
        }

        let i = self.segments.len() - 1;
        self.segments[i] = end;
    }

    /// Removes every point from it's segments array.
    /// Must call `init_path` after to star a new path!
    pub fn reset(
        &mut self,
    ) {
        self.segments.clear();
    }

    /// Updates the mesh.
    /// Call this after setting segments.
    pub fn update(&self, mesh: &mut Mesh) {
        let points = self.segments.clone();
        let normals = vec![[1.0, 1.0, 1.0]; points.len()];
        let uvs = vec![[1.0, 1.0]; points.len()];

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }

    /// Spawns the wire path on a given position, with just one segment.
    pub fn spawn3d(
        start: Vec3,
        end: Vec3,
        color: [f32; 3],
        path_id: impl Component,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let mut wp = WirePath::new(color);
        wp.init_path(start, end);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(wp.get_mesh()),
                material: materials.add(Color::from(color).into()),
                ..default()
            },
            wp,
            path_id
        ));
    }

    fn get_mesh(&self) -> Mesh {
        let positions = self.segments.clone();
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
