use bevy::prelude::*;
use itertools::Itertools;

pub trait GlobalTransform {
    fn from_parent(&self, parent: &Transform) -> Self;
}

impl GlobalTransform for Transform {
    fn from_parent(&self, parent: &Transform) -> Self {
        // This isn't correct, but will let it be for now

        let translation = parent.rotation.mul_vec3(self.translation) + parent.translation;
        // let rotation = parent.rotation + parent.rotation;

        // println!("parent {}, this {}", parent.rotation, rotation);

        Transform {
            translation,
            rotation: parent.rotation,
            scale: self.scale,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Line {
    pub org: Vec3,
    pub dir: Vec3,
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

pub struct Plane {
    pub p: Vec3,
    pub normal: Vec3,
}

impl Plane {
    pub fn from_triangle(triangle: Triangle) -> Self {
        let (a, b, c) = (triangle.a, triangle.b, triangle.c);
        Self {
            p: a,
            normal: (b - a).cross(c - a).normalize_or_zero(),
        }
    }
}

/// Calculates the collision point with the plane formed by triangle abc,
/// considering current position and rotation.
pub fn get_collision_point(line: Line, triangle: Triangle) -> Option<Vec3> {
    const EPSILON: f32 = 0.0000001;

    let p = Plane::from_triangle(triangle);

    // parallel or not touching
    let norline = p.normal.dot(line.dir);
    if norline.abs() < EPSILON {
        return None;
    }

    let t = (p.normal.dot(p.p) - p.normal.dot(line.org)) / norline;
    Some(line.org + (line.dir * t))
}

/// Checks whether a point is inside a triangle or not.
pub fn point_inside_triangle(p: Vec3, triangle: Triangle) -> bool {
    let u = triangle.b - triangle.a;
    let v = triangle.c - triangle.a;
    let w = p - triangle.a;

    let uu = u.dot(u);
    let uv = u.dot(v);
    let vv = v.dot(v);
    let wu = w.dot(u);
    let wv = w.dot(v);
    let uv2uuvv = (uv * uv) - (uu * vv);

    let alpha = ((uv * wv) - (vv * wu)) / uv2uuvv;
    let beta = ((uv * wu) - (uu * wv)) / uv2uuvv;

    alpha >= 0.0 && beta >= 0.0 && alpha + beta <= 1.0
}

/// Distance collision (spherical collision)
pub trait DistanceCollider {
    fn collides_with_dist(
        &self,
        object: &impl DistanceCollision,
        self_transform: &Transform,
        object_transform: &Transform,
    ) -> bool;

    /// Generic Distance-Distance collision method.
    fn dist_with_dist_collision(
        a: &impl DistanceCollision,
        b: &impl DistanceCollision,
        a_transform: &Transform,
        b_transform: &Transform,
    ) -> bool {
        let a_pos = a.get_pos(a_transform);
        let b_pos = b.get_pos(b_transform);
        a_pos.distance(b_pos) < (a.get_radius() + b.get_radius())
    }
}

pub trait DistanceCollision {
    fn get_pos(&self, transform: &Transform) -> Vec3 {
        transform.translation
    }

    fn get_radius(&self) -> f32;

    fn draw_sphere(&self, transform: &Transform, color: Color, gizmos: &mut Gizmos) {
        gizmos.sphere(
            self.get_pos(transform),
            Quat::IDENTITY,
            self.get_radius(),
            color,
        );
    }
}

/// Mesh collision (convex hull collision)
pub trait MeshCollision {
    fn get_hull(&self, mesh: &Mesh, transform: &Transform) -> Vec<Triangle> {
        // println!("{:?}", mesh.primitive_topology());

        let vertices: Vec<Vec3> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .iter()
            .map(|s| {
                let v = transform.rotation.mul_vec3(Vec3::from_array(*s));
                v + transform.translation
            })
            .collect();

        mesh.indices()
            .unwrap()
            .iter()
            .chunks(3)
            .into_iter()
            .map(|mut chunk| Triangle {
                a: vertices[chunk.next().unwrap()],
                b: vertices[chunk.next().unwrap()],
                c: vertices[chunk.next().unwrap()],
            })
            .collect()
    }
}

pub trait RayCollider {
    fn collides_with_mesh(
        &mut self,
        self_transform: &Transform,
        object: &impl MeshCollision,
        object_mesh: &Mesh,
        object_transform: &Transform,
    ) -> bool;
}

#[cfg(test)]
mod geometric_primitive_tests {
    use super::*;

    fn cmp_float(a: f32, b: f32) {
        const ERR: f32 = 0.00001;
        assert!(
            (a - b).abs() < ERR,
            "a: {} not similar enough to b: {}",
            a,
            b
        );
    }

    fn cmp_vec3(a: Vec3, b: Vec3) {
        cmp_float(a.x, b.x);
        cmp_float(a.y, b.y);
        cmp_float(a.z, b.z);
    }

    #[test]
    fn test_plane_from_triangle_1() {
        let triangle = Triangle {
            a: Vec3::new(0.0, -1.0, -1.0),
            b: Vec3::new(0.0, -1.0, 1.0),
            c: Vec3::new(0.0, 1.0, 0.0),
        };
        let plane = Plane::from_triangle(triangle);

        assert_eq!(plane.p, Vec3::new(0.0, -1.0, -1.0));
        cmp_vec3(plane.normal, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn test_plane_from_triangle_2() {
        let triangle = Triangle {
            a: Vec3::new(1.0, -1.0, 1.0),
            b: Vec3::new(0.0, -1.0, 1.0),
            c: Vec3::new(2.0, 1.0, 3.0),
        };
        let plane = Plane::from_triangle(triangle);

        assert_eq!(plane.p, Vec3::new(1.0, -1.0, 1.0));
        cmp_vec3(plane.normal, Vec3::new(0.0, 0.707107, -0.707107));
    }
}

#[cfg(test)]
mod distance_collision_tests {
    use super::*;

    struct A {
        pos: Vec3,
        radius: f32,
    }

    impl A {
        fn new(pos: Vec3, radius: f32) -> Self {
            A { pos, radius }
        }

        fn t(&self) -> Transform {
            Transform::from_translation(self.pos)
        }
    }

    impl DistanceCollision for A {
        fn get_radius(&self) -> f32 {
            self.radius
        }
    }

    impl DistanceCollider for A {
        fn collides_with_dist(
            &self,
            object: &impl DistanceCollision,
            self_transform: &Transform,
            object_transform: &Transform,
        ) -> bool {
            Self::dist_with_dist_collision(self, object, self_transform, object_transform)
        }
    }

    #[test]
    fn test_distance_collision() {
        let a = A::new(Vec3::ZERO, 1.0);
        let b = A::new(Vec3::ONE, 1.0);

        assert!(a.collides_with_dist(&b, &a.t(), &b.t()))
    }

    #[test]
    fn test_distance_collision_2() {
        let a = A::new(Vec3::ZERO, 1.0);
        let b = A::new(Vec3::new(3.0, 0.0, 0.0), 1.0);

        assert!(!a.collides_with_dist(&b, &a.t(), &b.t()))
    }
}

#[cfg(test)]
mod geometry_functions_tests {
    use super::*;

    #[test]
    fn test_get_collision_point_1() {
        let line = Line {
            org: Vec3::new(-1.0, 0.0, 0.0),
            dir: Vec3::X,
        };
        let triangle = Triangle {
            a: Vec3::new(0.0, -1.0, -1.0),
            b: Vec3::new(0.0, -1.0, 1.0),
            c: Vec3::new(0.0, 1.0, 0.0),
        };

        assert_eq!(get_collision_point(line, triangle), Some(Vec3::ZERO))
    }

    #[test]
    fn test_get_collision_point_2() {
        let line = Line {
            org: Vec3::new(-1.0, 0.0, 0.0),
            dir: Vec3::X,
        };
        let triangle = Triangle {
            a: Vec3::new(0.0, -1.0, -1.0),
            b: Vec3::new(0.0, -1.0, 1.0),
            c: Vec3::new(0.0, 1.0, 0.0),
        };

        assert_eq!(get_collision_point(line, triangle), Some(Vec3::ZERO))
    }

    #[test]
    fn test_get_collision_point_no_collision() {
        let line = Line {
            org: Vec3::new(-2.0, 0.0, 0.0),
            dir: Vec3::new(0.0, 0.5, 0.0),
        };
        let triangle = Triangle {
            a: Vec3::new(0.0, -1.0, -1.0),
            b: Vec3::new(0.0, -1.0, 1.0),
            c: Vec3::new(0.0, 1.0, 0.0),
        };

        assert_eq!(get_collision_point(line, triangle), None)
    }

    #[test]
    fn test_point_inside_triangle_1() {
        let triangle = Triangle {
            a: Vec3::new(0.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, -1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        };

        assert!(point_inside_triangle(Vec3::ZERO, triangle))
    }

    #[test]
    fn test_point_inside_triangle_2() {
        let triangle = Triangle {
            a: Vec3::new(0.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, -1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        };

        assert!(point_inside_triangle(Vec3::new(0.1, 0.0, 0.24), triangle))
    }

    #[test]
    fn test_point_outside_triangle() {
        let triangle = Triangle {
            a: Vec3::new(0.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, -1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        };

        assert!(!point_inside_triangle(Vec3::new(4.0, 0.0, 0.0), triangle))
    }
}
