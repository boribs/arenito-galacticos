use bevy::prelude::*;

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

/// Distance collision (spherical collision)
pub trait WithDistanceCollision {
    fn collides_with_dist(
        &self,
        object: &impl WithDistanceCollision,
        self_transform: &Transform,
        object_transform: &Transform,
    ) -> bool {
        let self_pos = self.get_pos(self_transform);
        let object_pos = object.get_pos(object_transform);
        self_pos.distance(object_pos) < (self.get_radius() + object.get_radius())
    }

    fn draw_sphere(&self, transform: &Transform, color: Color, gizmos: &mut Gizmos) {
        gizmos.sphere(
            self.get_pos(transform),
            Quat::IDENTITY,
            self.get_radius(),
            color,
        );
    }

    fn get_pos(&self, transform: &Transform) -> Vec3 {
        transform.translation
    }

    fn get_radius(&self) -> f32;
}

#[allow(unused)]
/// Mesh collision (convex hull collision)
pub trait WithMeshCollision {
    fn get_hull(&self, mesh: &Mesh, transform: &Transform) -> Vec<Vec3> {
        // println!("{:?}", mesh.primitive_topology());

        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .iter()
            .map(|s| {
                let v = transform.rotation.mul_vec3(Vec3::from_array(*s));
                v + transform.translation
            })
            .collect()
    }
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

    impl WithDistanceCollision for A {
        fn get_radius(&self) -> f32 {
            self.radius
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
