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
        let b = Vec2::new(0., height);
        // height
        let total_height = (b - a).length();
        let body_height = total_height - head_height;
        // width
        // body width in both directions, from point a
        let body_width = width / 4.0;

        let vertices = vec![
            [a.x + body_width, a.y, 0.0],
            [a.x + body_width, a.y + body_height, 0.0],
            [a.x + width, a.y + body_height, 0.0],
            [b.x, b.y, 0.0],
            [a.x - width, a.y + body_height, 0.0],
            [a.x - body_width, a.y + body_height, 0.0],
            [a.x - body_width, a.y, 0.0],
            [a.x + body_width, a.y, 0.0],
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
