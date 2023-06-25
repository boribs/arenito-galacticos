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
