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
    fn get_type(&self) -> CollisionType;
    fn collides_with(&self, object: &impl WithCollision) -> bool {
        match object.get_type() {
            CollisionType::Distance => self.collides_with_distance(object),
            CollisionType::Mesh => self.collides_with_mesh(object),
        }
    }
    fn collides_with_distance(&self, object: &impl WithCollision) -> bool;
    fn collides_with_mesh(&self, object: &impl WithCollision) -> bool;
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
    fn get_type(&self) -> CollisionType {
        CollisionType::Distance
    }

    fn collides_with_distance(&self, object: &impl WithCollision) -> bool {
        todo!()
    }

    fn collides_with_mesh(&self, object: &impl WithCollision) -> bool {
        todo!()
    }
}

/// Convex Hull collider.
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
    fn get_type(&self) -> CollisionType {
        CollisionType::Mesh
    }

    fn collides_with_distance(&self, object: &impl WithCollision) -> bool {
        todo!()
    }

    fn collides_with_mesh(&self, object: &impl WithCollision) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    fn test_distance_collision() {
        let a = DistanceCollider::new(10.0, &Vec3::ZERO);
        let b = DistanceCollider::new(10.0, &Vec3::ONE);

        assert!(a.collides_with(&b))
    }
}
