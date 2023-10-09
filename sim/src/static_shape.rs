use crate::sensor::ImageProcessor;
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
    // Horizontal view angle, in degrees
    ha: f32,
    // Vertical view angle, in degrees
    va: f32,
}

impl CameraPrism {
    pub fn new(ha: f32, va: f32) -> Self {
        Self {
            ha: ha.to_radians(),
            va: va.to_radians(),
        }
    }

    /// Calculates the points that limit the base of the prism.
    /// Considers the tip of the prism to be the origin (0, 0, 0).
    pub fn get_points(&self) -> Vec<Vec3> {
        // a, b, c and d are the vertices of the base of
        // the prism. The prism has it's top on `o` and
        // is inclinated such that the top and the center
        // of the base are aligned on the x+ axis.
        // ...............................................
        // ...............................---C............
        // ..........................----....|.-.D........
        // .....................----______...|...|........
        // ................----___...........|...|........
        // .x+.----->....O...................|.#.|........
        // ................----..............|...|........
        // ...................__----.........|...|........
        // ........................__----....|...|........
        // ...........................____---A.-.|........
        // ................................._____B........
        // ...............................................
        //
        // since we're looking for the most basic prism, it's sides
        // from the tip, to each corner of the base are of length 1.
        //
        // ...............................................
        // ....................................---#...-...
        // .........................----------....|...|...
        // ..............-----------..............|..Y/Z..
        // ........------.........................|...|...
        // .....#---------------------------------#...-...
        // ...............................................
        // ....|-----------------X----------------|.......
        // ...............................................
        //
        // given that sin = op / hip and cos = ad / hip, the coordinates
        // of the points are easy to infer.

        let (ha, va) = (self.ha / 2.0, self.va / 2.0);
        vec![
            Vec3::new(ha.cos(), va.sin(), -ha.sin()),  // a
            Vec3::new(ha.cos(), va.sin(), ha.sin()),   // b
            Vec3::new(ha.cos(), -va.sin(), ha.sin()),  // c
            Vec3::new(ha.cos(), -va.sin(), -ha.sin()), // d
        ]
    }

    /// Returns a prism with angles from a CameraArea.
    pub fn from_cam(camera_area: &CameraArea) -> Self {
        Self {
            ha: camera_area.ha,
            va: camera_area.va,
        }
    }
}

impl Default for CameraPrism {
    fn default() -> Self {
        CameraPrism::new(81.0, 65.0)
    }
}

impl From<CameraPrism> for Mesh {
    /// Creates a default CameraPrism looking in the default Z- direction.
    fn from(camera_prism: CameraPrism) -> Self {
        let points = camera_prism.get_points();
        let vertices = vec![
            Vec3::ZERO,
            points[0].clone(),
            Vec3::ZERO,
            points[1].clone(),
            Vec3::ZERO,
            points[2].clone(),
            Vec3::ZERO,
            points[3].clone(),
            points[0].clone(),
            points[1].clone(),
            points[1].clone(),
            points[2].clone(),
            points[2].clone(),
            points[3].clone(),
            points[3].clone(),
            points[0].clone(),
        ];
        let normals = vec![[1.0, 1.0, 1.0]; vertices.len()];
        let uvs = vec![[1.0, 1.0]; vertices.len()];

        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

/// Visualization of the area visible by Arneito's camera.
#[derive(Component)]
pub struct CameraArea {
    // Camera's position
    pos: Vec3,
    // Horizontal view angle, in degrees
    ha: f32,
    // Vertical view angle, in degrees
    va: f32,
    // Camera's vertical rotation
    alpha: f32,
}

impl CameraArea {
    pub fn new(pos: Vec3, ha: f32, va: f32, alpha: f32) -> Self {
        Self {
            pos,
            ha: ha.to_radians(),
            va: va.to_radians(),
            alpha: alpha.to_radians(),
        }
    }

    /// Calculates the points that limit the camera's visible area.
    pub fn area_points(&self) -> Vec<Vec3> {
        // A and B are the closest points to the camera
        // in right-to-left order.
        // C and D are in left-to-right order, further away.
        //
        //    C         B
        //
        //      D     A
        //
        //        cam

        let q = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, self.alpha);
        let mut points = CameraPrism::from_cam(self).get_points();

        for i in 0..points.len() {
            // rotate each point and move to correct position
            let p = q.mul_vec3(points[i]) + self.pos;

            // since it's about a 3d line, we have to consider two planes xy and xz.
            //
            // starting with xy: the line equation goes: y - y_0 = m(x - x_0)
            // we know that y = 0, because we want to know where it reaches the ground:
            // (0) - y_0 = m(x - x_0)
            // -y_0 = mx - mx_0
            //
            // and we want to find x:
            // mx = mx_0 - y_0
            // x = x_0 - (y_0 / m)
            //
            // we also know that the initial position (x_0 and y_0) is the camera's,
            // so, we can re-write the equation as:
            // x = pos.x - (pos.y / m)
            //
            // now the xz plane: based on the same equation, but replacing y by z,
            // this plane's line equation is: z - z_0 = m(x - x_0)
            // since we already know x, we just have to calculate this plane's slope
            // and substitute the rest:
            // z = m(x - x_0) + z_0

            let mxy = (p.y - self.pos.y) / (p.x - self.pos.x); // xy slope
            let mxz = (p.z - self.pos.z) / (p.x - self.pos.x); // xz slope

            let x = self.pos.x - (self.pos.y / mxy);
            points[i] = Vec3::new(x, 0.0, mxz * (x - self.pos.x) + self.pos.z);
        }

        points
    }

    /// Creates a CameraArea instance taking camera data from ArenitoCamData.
    pub fn from_cam_data(cam_data: &ImageProcessor) -> Self {
        Self {
            pos: cam_data.offset.clone(),
            ha: cam_data.ha.to_radians(),
            va: cam_data.va.to_radians(),
            alpha: cam_data.alpha.to_radians(),
        }
    }
}

impl Default for CameraArea {
    fn default() -> Self {
        Self::new(Vec3::new(0.75, 1.3, 0.0), 45.0, 45.0, -40.0)
    }
}

impl From<CameraArea> for Mesh {
    fn from(cam_area: CameraArea) -> Self {
        let mut points = cam_area.area_points();
        points.push(points[0].clone());

        let normals = vec![[1.0, 1.0, 1.0]; 5];
        let uvs = vec![[1.0, 1.0]; 5];

        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
