use bevy::prelude::*;

/// Every collidable object must implement some collision trait

/// Distance collision (spherical collision)
pub trait WithDistanceCollision {
    fn collides_with_dist(&self, object: &impl WithDistanceCollision) -> bool {
        self.get_pos().distance(object.get_pos()) < self.get_radius() + object.get_radius()
    }
    fn draw_sphere(&self, color: Color, gizmos: &mut Gizmos) {
        gizmos.sphere(self.get_pos(), Quat::IDENTITY, self.get_radius(), color);
    }
    fn get_pos(&self) -> Vec3;
    fn get_radius(&self) -> f32;
}

#[allow(unused)]
/// Mesh collision (convex hull collision)
pub trait WithMeshCollision {
    fn collides_with_mesh(&self, object: &impl WithMeshCollision) -> bool {
        todo!()
    }
    fn get_hull(&self) -> Vec<Vec3>;
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    struct A {
        pos: Vec3,
        radius: f32,
    }

    impl A {
        fn new(pos: Vec3, radius: f32) -> Self {
            A { pos, radius }
        }
    }

    impl WithDistanceCollision for A {
        fn get_pos(&self) -> Vec3 {
            self.pos
        }
        fn get_radius(&self) -> f32 {
            self.radius
        }
    }

    impl WithMeshCollision for A {
        fn get_hull(&self) -> Vec<Vec3> {
            Vec::new()
        }
    }

    struct B;

    impl WithMeshCollision for B {
        fn get_hull(&self) -> Vec<Vec3> {
            Vec::new()
        }
    }

    #[test]
    fn test_distance_collision() {
        let a = A::new(Vec3::ZERO, 1.0);
        let b = A::new(Vec3::ONE, 1.0);

        assert!(a.collides_with_dist(&b))
    }

    #[test]
    fn test_distance_collision_2() {
        let a = A::new(Vec3::ZERO, 1.0);
        let b = A::new(Vec3::new(3.0, 0.0, 0.0), 1.0);

        assert!(!a.collides_with_dist(&b))
    }

    #[test]
    #[should_panic]
    fn test_distance_with_mesh_collision() {
        let a = A::new(Vec3::ZERO, 1.0);
        let b = B;

        assert!(a.collides_with_mesh(&b))
    }
}
