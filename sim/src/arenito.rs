use bevy::prelude::*;
use std::f32::consts::TAU;

const ACCEL_SPEED: f32 = 4.0;
const ROT_SPEED: f32 = 1.5;
const FRIC_K: f32 = 0.5;

/// Component used as an identifier for the different
/// body parts in Arenito.
#[derive(Component)]
pub enum BodyPart {
    Frame,
    LeftWheel,
    RightWheel,
}

/// Describes Arenito's state.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ArenitoState {
    LEFT = -1,
    RIGHT = 1,
    FORWARD,
    STILL,
}

/// Arenito is the main component of this simulation.
///
/// It's responsible of both visual and "logical" updates of position,
/// velocity, acceleration and rotation.
/// Those attributes will be important when simulating the sensors.
#[derive(Resource)]
pub struct Arenito {
    pub center: Vec3,
    pub vel: Vec3,
    pub acc: Vec3,
    pub state: ArenitoState,
    pub look_angle: f32, // on the y axis
}

impl Arenito {
    /// Returns an empty, non-spawned Arenito.
    pub fn new() -> Self {
        Arenito {
            center: Vec3::new(0.0, 0.5, 0.0),
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            look_angle: 0.0,
            state: ArenitoState::STILL,
        }
    }

    /// Spawns Arenito (body cube and wheels) into the scene.
    ///
    /// Arenito's model is a cube (parent) with four wheels (children).
    /// This is to preserve positional rotation (not having to manually
    /// rotate each wheel), facilitating significantly rotating the wheels
    /// on the z axis when moving forward or rotating.
    pub fn spawn(
        &self,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
    ) {
        commands
            .spawn((
                PbrBundle {
                    mesh: asset_server.load("arenito.obj"),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(self.center.x, self.center.y, self.center.z),
                    ..default()
                },
                BodyPart::Frame,
            ))
            .with_children(|parent| {
                let t = self.center + Vec3::new(0.5, -0.5, 0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    BodyPart::RightWheel,
                ));
                let t = self.center + Vec3::new(-0.5, -0.5, 0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    BodyPart::RightWheel,
                ));
                let t = self.center + Vec3::new(0.5, -0.5, -0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    BodyPart::LeftWheel,
                ));
                let t = self.center + Vec3::new(-0.5, -0.5, -0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    BodyPart::LeftWheel,
                ));
            });
    }

    /// Sets the acceleration to "advance acceleration".
    pub fn forward(&mut self) {
        if self.state != ArenitoState::STILL && self.state != ArenitoState::FORWARD {
            return;
        }

        let (sin, cos) = self.look_angle.sin_cos();
        self.acc = Vec3::new(cos, 0.0, sin) * ACCEL_SPEED;
        self.state = ArenitoState::FORWARD;
    }

    /// Sets Arenito in "rotation mode" - sets the acceleration
    /// to the correct values.
    pub fn rotate(&mut self, dir: ArenitoState) {
        if self.state != ArenitoState::STILL && self.state != dir {
            return;
        }

        self.acc = Vec3::ONE * ROT_SPEED;
        self.state = dir;
    }

    /// Resets the state of Arenito.
    /// This includes despawning and spawning the models. It was easier than
    /// resetting everything to it's original state.
    pub fn reset(
        &mut self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        body_part_query: &Query<(&mut Transform, &BodyPart, Entity, With<BodyPart>)>,
    ) {
        self.center = Vec3::new(0.0, 0.5, 0.0);
        self.acc = Vec3::ZERO;
        self.vel = Vec3::ZERO;
        self.state = ArenitoState::STILL;
        self.look_angle = 0.0;

        body_part_query.for_each(|e| {
            commands.entity(e.2).despawn();
        });

        self.spawn(commands, materials, asset_server);
    }

    /// Applies the movement given some delta time.
    /// This is both in "logical units" (the real units Arenito is actually at)
    /// and visually (whatever Bevy's renderer needs to update what we see).
    ///
    /// This big method considers Arenito's state, updating both the main body's
    /// position (the cube) and the wheels' rotation (the direction changes depending
    /// on whether it's advancing forward or rotating).
    ///
    /// It also updates Arenito's velocity and acceleration.
    ///
    /// * `delta_ms` - time delta between this and the last frame this was called.
    /// * `body_part_query` - Bevy's way of finding elements.
    pub fn update(
        &mut self,
        delta_ms: u128,
        body_part_query: Query<(&mut Transform, &BodyPart, Entity, With<BodyPart>)>,
    ) {
        let vec = self.update_pos(delta_ms);
        self.update_model(vec, body_part_query);
    }

    /// Updates Arenito's position given some time in ms (`delta_ms`).
    /// This method is suposed to be called every frame, where delta_ms
    /// is the time between this frame's render and the previous one.
    ///
    /// Depending on Arenito's state it will:
    ///   - Move forward
    ///   - Rotate
    fn update_pos(&mut self, delta_ms: u128) -> (Vec3, f32) {
        let delta: f32 = delta_ms as f32 / 1000.0;

        // Friction needs to be calculated every frame, because its a vector
        // that directly opposes movement.
        let fric = self.acc.normalize_or_zero() * -1.0 * FRIC_K;

        self.acc += fric; // Sum it because it's already inverted
        self.vel = (self.acc * delta) + self.vel;
        // TODO: Cap top speed

        // If the force of friction is bigger than Arenito's forward acceleration
        // and the computation continues as is, Arenito will move backwards!
        // If Arenito is unable to overpower friction, then it should stop.
        if self.acc.length() < FRIC_K {
            self.vel = Vec3::ZERO;
            self.acc = Vec3::ZERO;
            self.state = ArenitoState::STILL;
        }

        // Highschool physics: Distance = v_0 * t + (0.5 * a * t^2)
        // This is also valid when velocity (v_0) and acceleratoin (a)
        // are both vectors.
        let d = (self.vel * delta) + (0.5 * self.acc * delta * delta);
        let dl = d.length();

        if self.state == ArenitoState::FORWARD {
            self.center += d;

            return (d, dl);
        } else {
            let theta = dl * self.state as isize as f32;
            self.look_angle = (self.look_angle + theta) % TAU;

            return (d, theta);
        }
    }

    /// Updates Arenito's rendered model.
    /// That's the main cube and the wheels. They are moved according to the values inside
    /// `vec` tuple:
    ///   * `Vec3` is the distance vector: how much has moved since the last frame.
    ///   * `f32` is either the length of the vector (if Arenito moved forward) or
    ///           the rotation delta (how much arenito rotated since the last frame).
    ///           This value is to rotate the wheels.
    fn update_model(
        &self,
        vec: (Vec3, f32),
        mut body_part_query: Query<(&mut Transform, &BodyPart, Entity, With<BodyPart>)>,
    ) {
        // Saving different body parts to their own variable.
        // Each body part behaves differently.
        let mut body = Vec::<Mut<'_, Transform>>::with_capacity(1);
        let mut left_wheels = Vec::<Mut<'_, Transform>>::with_capacity(2);
        let mut right_wheels = Vec::<Mut<'_, Transform>>::with_capacity(2);

        for body_part in &mut body_part_query {
            match body_part.1 {
                BodyPart::LeftWheel => {
                    left_wheels.push(body_part.0);
                }
                BodyPart::RightWheel => {
                    right_wheels.push(body_part.0);
                }
                BodyPart::Frame => {
                    body.push(body_part.0);
                }
            }
        }

        // Since body is only one element, shadow it out of the vector!
        let body = &mut body[0];
        let (d, l) = vec;

        match self.state {
            ArenitoState::FORWARD => {
                body.translation += d;

                for wheel in &mut left_wheels {
                    wheel.rotate_local_z(-l);
                }
                for wheel in &mut right_wheels {
                    wheel.rotate_local_z(-l);
                }
            }
            ArenitoState::RIGHT | ArenitoState::LEFT => {
                body.translation -= self.center;
                body.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-l));
                body.translation += self.center;

                for wheel in &mut left_wheels {
                    wheel.rotate_local_z(-l);
                }
                for wheel in &mut right_wheels {
                    wheel.rotate_local_z(l);
                }
            }
            _ => {}
        }
    }

    /// Prints the current stats of Arenito.
    pub fn log(&self) -> String {
        format!(
            "c: {} acc: {} vel: {} º: {} - {:?}",
            self.center, self.acc, self.vel, self.look_angle, self.state
        )
    }
}

#[cfg(test)]
mod arenito_tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    const F32_DIFF: f32 = 0.001;

    /// Compares vectors' values.
    /// Considers "equal" values within a difference of `F32_DIFF`.
    fn cmp_vec(a: Vec3, b: Vec3) {
        assert!((a.x - b.x).abs() < F32_DIFF, "x: {} != {}", a.x, b.x);
        assert!((a.y - b.y).abs() < F32_DIFF, "y: {} != {}", a.y, b.y);
        assert!((a.z - b.z).abs() < F32_DIFF, "z: {} != {}", a.z, b.z);
    }

    // ------------------------------------------------------------
    // The following tests are to test Arenito's movement a single
    // frame forward, from absolute rest on a flat surface.

    #[test]
    fn accelerated_movement_positive_x() {
        let mut arenito = Arenito::new();
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(3.5, 0.0, 0.0);
        let expected_vel = Vec3::new(0.056, 0.0, 0.0);
        let expected_center = Vec3::new(0.001344, 0.5, 0.0);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_positive_xz() {
        let mut arenito = Arenito::new();
        arenito.look_angle = FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(2.47487, 0.0, 2.47487);
        let expected_vel = Vec3::new(0.03959, 0.0, 0.03959);
        let expected_center = Vec3::new(0.00095, 0.5, 0.00095);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_positive_z() {
        let mut arenito = Arenito::new();
        arenito.look_angle = FRAC_PI_2;
        arenito.forward();
        arenito.update_pos(16);

        // most zeros aren't actually zero, but very close
        let expected_acc = Vec3::new(0.0, 0.0, 3.5);
        let expected_vel = Vec3::new(0.0, 0.0, 0.056);
        let expected_center = Vec3::new(0.0, 0.5, 0.001344);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_negative_x_positive_z() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 3.0 * FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(-2.47487, 0.0, 2.47487);
        let expected_vel = Vec3::new(-0.03959, 0.0, 0.03959);
        let expected_center = Vec3::new(-0.0009, 0.5, 0.0009);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_negative_x() {
        let mut arenito = Arenito::new();
        arenito.look_angle = PI;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(-3.5, 0.0, 0.0);
        let expected_vel = Vec3::new(-0.056, 0.0, 0.0);
        let expected_center = Vec3::new(-0.001344, 0.5, 0.0);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_negative_x_negative_z() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 5.0 * FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(-2.47487, 0.0, -2.47487);
        let expected_vel = Vec3::new(-0.03959, 0.0, -0.03959);
        let expected_center = Vec3::new(-0.0009, 0.5, -0.0009);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_negative_z() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 3.0 * FRAC_PI_2;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(0.0, 0.0, -3.5);
        let expected_vel = Vec3::new(0.0, 0.0, -0.056);
        let expected_center = Vec3::new(0.0, 0.5, -0.001344);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_negative_z_positive_x() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 7.0 * FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(2.47487, 0.0, -2.47487);
        let expected_vel = Vec3::new(0.03959, 0.0, -0.03959);
        let expected_center = Vec3::new(0.00095, 0.5, -0.00095);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_1() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 0.1234;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(3.47338, 0.0, 0.43080);
        let expected_vel = Vec3::new(0.05557, 0.0, 0.00689);
        let expected_center = Vec3::new(0.0013, 0.5, 0.00016);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_2() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 0.38;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(3.25032, 0.0, 1.29822);
        let expected_vel = Vec3::new(0.05200, 0.0, 0.020771);
        let expected_center = Vec3::new(0.00124, 0.5, 0.00049);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_3() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 4.7551;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(0.14944, 0.0, -3.49680);
        let expected_vel = Vec3::new(0.00239, 0.0, -0.055948);
        let expected_center = Vec3::new(0.0, 0.5, -0.0013);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_4() {
        let mut arenito = Arenito::new();
        arenito.look_angle = -6.1362;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(3.46229, 0.0, 0.51233);
        let expected_vel = Vec3::new(0.0553, 0.0, 0.008197);
        let expected_center = Vec3::new(0.00132, 0.5, 0.00019);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_5() {
        let mut arenito = Arenito::new();
        arenito.look_angle = -0.713244;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(2.6468, 0.0, -2.29001);
        let expected_vel = Vec3::new(0.042349, 0.0, -0.03664);
        let expected_center = Vec3::new(0.001016, 0.5, -0.00087);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_6() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 3.70245;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(-2.9637, 0.0, -1.8617);
        let expected_vel = Vec3::new(-0.04742, 0.0, -0.02978);
        let expected_center = Vec3::new(-0.00113, 0.5, -0.00071);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_7() {
        let mut arenito = Arenito::new();
        arenito.look_angle = -1.4037;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(0.58178, 0.0, -3.45130);
        let expected_vel = Vec3::new(0.00930, 0.0, -0.05522);
        let expected_center = Vec3::new(0.00022, 0.5, -0.00132);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_8() {
        let mut arenito = Arenito::new();
        arenito.look_angle = -1.4037;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(0.58178, 0.0, -3.45130);
        let expected_vel = Vec3::new(0.00930, 0.0, -0.05522);
        let expected_center = Vec3::new(0.00022, 0.5, -0.00132);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_9() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 1.65394;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(-0.29068, 0.0, 3.487908);
        let expected_vel = Vec3::new(-0.00465, 0.0, 0.055806);
        let expected_center = Vec3::new(-0.00011, 0.5, 0.001339);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    #[test]
    fn accelerated_movement_random_angle_10() {
        let mut arenito = Arenito::new();
        arenito.look_angle = 0.52525;
        arenito.forward();
        arenito.update_pos(16);

        let expected_acc = Vec3::new(3.02817, 0.0, 1.75502);
        let expected_vel = Vec3::new(0.04845, 0.0, 0.02808);
        let expected_center = Vec3::new(0.00116, 0.5, 0.00067);

        cmp_vec(arenito.vel, expected_vel);
        cmp_vec(arenito.acc, expected_acc);
        cmp_vec(arenito.center, expected_center);
    }

    // TODO: stopping tests
    // TOOD: zero movement tests
    // TODO: max vel tests
}
