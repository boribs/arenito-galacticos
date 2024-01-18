use bevy::prelude::*;

// Collisions!
// Every collidable object has to have some way of identification.
pub enum CollisionType {
    Distance,
    Mesh,
    // Cylinder, // soon
}

/// Collision trait!
pub trait WithCollision {
    fn collides_with_distance(&self, object: &DistanceCollider) -> bool;
    fn collides_with_mesh(&self, object: &MeshCollider) -> bool;
}

/// Can also be refered to as "Shpere collider".
/// Checks the distance from this object to another.
pub struct DistanceCollider<'a> {
    radius: f32,
    center: &'a Vec3,
}

impl <'a>DistanceCollider<'a> {
    pub fn new(radius: f32, center: &'a Vec3) -> Self {
        DistanceCollider {
            radius,
            center
        }
    }
}

impl WithCollision for DistanceCollider<'_> {
    fn collides_with_distance(&self, object: &DistanceCollider) -> bool {
        self.center.distance(*object.center) < self.radius + object.radius
    }

    #[allow(unused)]
    fn collides_with_mesh(&self, object: &MeshCollider) -> bool {
        todo!("distance-vs-mesh collision")
    }
}

/// Convex Hull collider.
#[allow(unused)]
pub struct MeshCollider<'a> {
    center: &'a Vec3,
    hull: Vec<Vec3>,
}

impl <'a>MeshCollider<'a> {
    pub fn new(center: &'a Vec3, hull: Vec<Vec3>) -> Self {
        MeshCollider { center, hull }
    }
}

impl WithCollision for MeshCollider<'_> {
    #[allow(unused)]
    fn collides_with_distance(&self, object: &DistanceCollider) -> bool {
        todo!()
    }

    #[allow(unused)]
    fn collides_with_mesh(&self, object: &MeshCollider) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    fn test_distance_collision() {
        let a = DistanceCollider::new(1.0, &Vec3::ZERO);
        let b = DistanceCollider::new(1.0, &Vec3::ONE);

        assert!(a.collides_with_distance(&b))
    }

    #[test]
    fn test_distance_collision_2() {
        let a = DistanceCollider::new(1.0, &Vec3::ZERO);
        let v = Vec3::new(3.0, 0.0, 0.0);
        let b = DistanceCollider::new(1.0, &v);

        assert!(!a.collides_with_distance(&b))
    }
}
