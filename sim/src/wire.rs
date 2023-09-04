use bevy::{
    prelude::*,
    render::render_resource::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

/* ----------------------------Vec3::3d -> 2d---------------------------- */
pub trait To2D {
    fn to_2d(&self) -> Self;
}

impl To2D for Vec3 {
    fn to_2d(&self) -> Self {
        Vec3::new(self.x, self.z, 0.0)
    }
}

#[derive(Component, Copy, Clone)]
pub struct Wire3D {
    start: Vec3,
    end: Vec3,
}

impl Default for Wire3D {
    fn default() -> Self {
        Self {
            start: Vec3::ZERO,
            end: Vec3::new(1.0, 0.0, 0.0),
        }
    }
}

impl Wire3D {
    /// Creates a new 3D Wire component.
    /// This is supposed to be passed on spawn and then retrieved
    /// with a Query.
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Wire3D { start, end }
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

    /// Spawns a 3D Wire on a given position with another component.
    pub fn spawn(
        start: Vec3,
        end: Vec3,
        color: [f32; 3],
        component: impl Component,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        let w = Wire3D::new(start, end);
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

impl From<Wire3D> for Mesh {
    fn from(wire: Wire3D) -> Self {
        let Wire3D { start, end } = wire;

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

#[derive(Component, Copy, Clone)]
pub struct Wire2D {
    start: Vec3,
    end: Vec3,
}

impl Default for Wire2D {
    fn default() -> Self {
        Self {
            start: Vec3::ZERO,
            end: Vec3::new(1.0, 1.0, 0.0),
        }
    }
}

impl Wire2D {
    /// Creates a new Wire component.
    /// This is supposed to be passed on spawn and then retrieved
    /// with a Query.
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Wire2D { start, end }
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

    /// Spawns a 2D Wire on a given position with another component.
    pub fn spawn(
        start: Vec3,
        end: Vec3,
        color: [f32; 3],
        component: impl Component,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
    ) {
        let w = Wire2D::new(start, end);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(w.into())),
                material: materials.add(ColorMaterial::from(Color::from(color))),
                ..default()
            },
            w,
            component,
        ));
    }
}

impl From<Wire2D> for Mesh {
    fn from(wire: Wire2D) -> Self {
        let Wire2D { start, end } = wire;

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

/// This struct is to (visually) describe paths!
/// It's made of lines described in `WirePath::points`, where the first
/// point is the beginning, the second is the end of the first segment
/// and the beginning of the second, the third point is the end
/// of the second segment and so on.
#[derive(Component)]
pub struct WirePath {
    pub color: [f32; 3],
    pub points: Vec<Vec3>,
}

impl WirePath {
    /// Creates a new WirePath instance. This will be used to control the path.
    /// A path is created with new_segment
    pub fn new(color: [f32; 3]) -> Self {
        WirePath {
            color,
            points: Vec::new(),
        }
    }

    /// Initializes a path. Creates a segment from `start` to `end`.
    /// This method must be called once at the beginning of the path.
    /// Subsecuent calls with an already initialized path will result in a panic.
    pub fn init_path(&mut self, start: Vec3, end: Vec3) {
        if !self.points.is_empty() {
            panic!("This method must be called only once!");
        }

        self.points.push(start);
        self.points.push(end);
    }

    /// Adds a new end point to the path, resulting in a new segment.
    pub fn append_segment(&mut self, end: Vec3) {
        if self.points.is_empty() {
            panic!("Must initialize a path before adding segments!");
        }

        self.points.push(end);
    }

    /// Deletes the last path segment.
    pub fn delete_last(&mut self) {
        if self.points.len() < 3 {
            return;
        }

        self.points.pop();
    }

    /// Updates the end position of the last segment.
    pub fn move_last(&mut self, end: Vec3) {
        if self.points.is_empty() {
            panic!("No segments in path!");
        }

        let i = self.points.len() - 1;
        self.points[i] = end;
    }

    /// Removes every point from it's segments array.
    /// Must call `init_path` after to star a new path!
    pub fn reset(&mut self) {
        self.points.clear();
    }

    /// Updates the mesh.
    /// Call this after setting segments.
    pub fn update(&self, mesh: &mut Mesh) {
        let points = self.points.clone();
        let normals = vec![[1.0, 1.0, 1.0]; points.len()];
        let uvs = vec![[1.0, 1.0]; points.len()];

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }

    /// Spawns the wire path on a given position and
    /// initializes it with just one segment.
    pub fn spawn(
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
            path_id,
        ));
    }

    /// Creates a mesh out of this path's segments.
    /// This method is not supposed to be called by the user.
    fn get_mesh(&self) -> Mesh {
        let positions = self.points.clone();
        let normals = vec![[1.0, 1.0, 1.0]; 2];
        let uvs = vec![[1.0, 1.0]; 2];

        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
