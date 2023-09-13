use bevy::{prelude::*, render::render_resource::*};

/// An arrow.
#[derive(Component)]
pub struct Arrow {
    /// Horizontal distance from center of the base to the furthest
    /// point of the arrow head.
    pub width: f32,
    /// Vertical distance from base to tip.
    pub height: f32,
    /// How much of the distance between points a and b is the head
    /// of the arrow.
    pub head_height: f32,
}

impl Arrow {
    pub fn new(width: f32, height: f32, head_height: f32) -> Self {
        Arrow {
            width,
            height,
            head_height,
        }
    }
}

impl Default for Arrow {
    fn default() -> Self {
        Arrow::new(15.0, 40.0, 20.0)
    }
}

impl From<Arrow> for Mesh {
    fn from(arrow: Arrow) -> Self {
        let Arrow {
            width,
            height,
            head_height,
        } = arrow;

        let a = Vec2::new(0., 0.);
        let b = Vec2::new(height, 0.);
        // height
        let total_height = (b - a).length();
        let body_height = total_height - head_height;
        // width
        // body width in both directions, from point a
        let body_width = width / 4.0;

        // vertices are of an arrow pointing in the x+ axis
        // ................................................
        // .........................X\.....................
        // .........................|..\...................
        // .........X---------------X....\.................
        // .........|......................\...............
        // .........a.......................Xb.............
        // .........|....................../...............
        // .........X---------------X..../.................
        // .........................|../...................
        // .........................X/.....................
        // ................................................
        let vertices = vec![
            [a.x, a.y + body_width, 0.0],
            [a.x + body_height, a.y + body_width, 0.0],
            [a.x + body_height, a.y + width, 0.0],
            [b.x, b.y, 0.0],
            [a.x + body_height, a.y - width, 0.0],
            [a.x + body_height, a.y - body_width, 0.0],
            [a.x, a.y - body_width, 0.0],
            [a.x, a.y + body_width, 0.0],
        ];
        let normals = vec![[1.0, 1.0, 1.0]; vertices.len()];
        let uvs = vec![[1.0, 1.0]; vertices.len()];

        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

/// Visual representation of the viewport of a virtual 3d Camera
#[derive(Component)]
pub struct CameraPrism {
    // How far the camera sees
    depth: f32,
    // Horizontal view angle, in degrees
    ha: f32,
    // Vertical view angle, in degrees
    va: f32,
}

impl CameraPrism {
    pub fn new(depth: f32, ha: f32, va: f32) -> Self {
        Self {
            depth,
            ha: ha.to_radians(),
            va: va.to_radians(),
        }
    }
}

impl Default for CameraPrism {
    fn default() -> Self {
        CameraPrism::new(10.0, 81.0, 65.0)
    }
}

impl From<CameraPrism> for Mesh {
    /// Creates a default CameraPrism looking in the default Z- direction.
    fn from(camera_prism: CameraPrism) -> Self {
        // a, b, c and d are the vertices of the base of
        // the prism. The prism has it's top on `o` and
        // is inclinated such that the top and the center
        // of the base are aligned on the z- axis.
        // ...............................................
        // ...............................---C............
        // ..........................----....|.-.D........
        // .....................----______...|...|........
        // ................----___...........|...|........
        // .z-.----->....O...................|.x.|........
        // ................----..............|...|........
        // ...................__----.........|...|........
        // ........................__----....|...|........
        // ...........................____---A.-.|........
        // ................................._____B........
        // ...............................................
        // ..............|--------depth--------|..........
        // ...............................................
        let dis = Vec3::new(
            (camera_prism.ha / 2.).tan(),
            (camera_prism.va / 2.).tan(),
            -camera_prism.depth,
        );
        let (a, b, c, d) = (
            dis,
            Vec3::new(dis.x, -dis.y, dis.z),
            Vec3::new(-dis.x, dis.y, dis.z),
            Vec3::new(-dis.x, -dis.y, dis.z),
        );

        let linelist = vec![
            Vec3::ZERO,
            a.clone(),
            Vec3::ZERO,
            b.clone(),
            Vec3::ZERO,
            c.clone(),
            Vec3::ZERO,
            d.clone(),
            // ---
            a.clone(),
            b.clone(),
            b.clone(),
            d.clone(),
            d.clone(),
            c.clone(),
            c.clone(),
            a.clone(),
        ];

        let normals = vec![[1.0, 1.0, 1.0]; linelist.len()];
        let uvs = vec![[1.0, 1.0]; linelist.len()];

        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, linelist);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
