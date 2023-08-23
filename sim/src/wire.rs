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

#[derive(Component)]
pub struct WirePathSegment(u32);

// TODO: Create WirePathSegmentLast and WirePathSegmentSecondLast
//       components to track path.
// TODO: When spawning wires, use Wire::get_bundle to (manually)
//       spawn wires, then add components.
// TODO: Use World::query to query second last path segment,
//       remove component. Then add second last to last and
//       last to the newest.

/// This struct is used to connect multiple wires to form a path.
/// It's intended use is to display the path Arenito travels.
#[derive(Resource)]
pub struct WirePath {
    pub segments: u32,
    pub color: [f32; 3],
    last_segment_end: Vec3,
}

impl WirePath {
    /// Creates a new WirePath instance. This will be used to control the path.
    /// A path is created with new_segment
    pub fn new(color: [f32; 3]) -> Self {
        WirePath {
            segments: 0,
            color,
            last_segment_end: Vec3::ZERO,
        }
    }

    /// Initializes a path. Creates a segment from `start` to `end`.
    /// This method must be called once at the beginning of the path.
    /// Subsecuent calls with an already initialized path will result in a panic.
    pub fn init_path(
        &mut self,
        start: Vec3,
        end: Vec3,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        if self.segments != 0 {
            panic!("This method must be called only once!");
        }

        Wire::spawn3d(
            start,
            end,
            self.color,
            WirePathSegment(0),
            commands,
            meshes,
            materials,
        );

        self.segments = 1;
        self.last_segment_end = end;
    }

    /// Spawns a new wire from the end position of the last path segment.
    pub fn append_segment(
        &mut self,
        end: Vec3,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        if self.segments == 0 {
            panic!("Must initialize a path before adding segments!");
        }

        Wire::spawn3d(
            self.last_segment_end,
            end,
            self.color,
            WirePathSegment(self.segments),
            commands,
            meshes,
            materials,
        );

        self.segments += 1;
        self.last_segment_end = end;
    }

    /// Deletes the last path segment.
    pub fn delete_last(
        &mut self,
        commands: &mut Commands,
        segment_query: Query<(&mut Wire, &WirePathSegment, Entity, &Handle<Mesh>)>,
    ) {
        if self.segments == 0 {
            return;
        }

        // find the last segment...
        for segment in &segment_query {
            if segment.1 .0 == self.segments - 1 {
                commands.entity(segment.2).despawn();
                break;
            }
        }

        self.segments -= 1;
    }

    /// Updates the end position of the last path segment.
    pub fn update_last(
        &mut self,
        end: Vec3,
        meshes: &mut ResMut<Assets<Mesh>>,
        mut segment_query: Query<(&mut Wire, &WirePathSegment, Entity, &Handle<Mesh>)>,
    ) {
        if self.segments == 0 {
            panic!("No segments in path!");
        }

        for mut segment in &mut segment_query {
            if segment.1 .0 == self.segments - 1 {
                segment.0.set_end(end);
                segment.0.update(meshes.get_mut(segment.3).unwrap());
                break;
            }
        }

        self.last_segment_end = end;
    }

    /// Despawns every wire of the path and restores segments and last_segment_end
    /// to their default values.
    /// Must call `init_path` after to star a new path!
    pub fn reset(
        &mut self,
        commands: &mut Commands,
        segment_query: Query<(&Wire, &WirePathSegment, Entity, &Handle<Mesh>)>,
    ) {
        for segment in &segment_query {
            commands.entity(segment.2).despawn();
        }

        self.segments = 0;
        self.last_segment_end = Vec3::ZERO;
    }
}

// TODO: Implement arrows
