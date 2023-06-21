use bevy::prelude::*;
use std::f32::consts::TAU;

const ACCEL_SPEED: f32 = 4.0;
const ROT_SPEED: f32 = 1.5;
const FRIC_K: f32 = 0.5;

#[derive(Component)]
pub enum BodyPart {
    Frame,
    LeftWheel,
    RightWheel,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ArenitoState {
    LEFT = -1,
    RIGHT = 1,
    FORWARD,
    STILL,
}

#[derive(Resource)]
pub struct Arenito {
    pub center: Vec3,
    pub vel: Vec3,
    pub acc: Vec3,
    pub state: ArenitoState,
    pub look_angle: f32, // on the y axis
    reset: bool,
}

impl Arenito {
    pub fn new() -> Self {
        Arenito {
            center: Vec3::new(0.0, 0.5, 0.0),
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            look_angle: 0.0,
            state: ArenitoState::STILL,
            reset: false,
        }
    }

    /// Spawns Arenito into the scene
    pub fn spawn(
        &self,
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
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
        if self.state != ArenitoState::STILL && self.state != ArenitoState::FORWARD
        {
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

    /// Sets the state to reset on the next call to Arenito::update().
    pub fn reset(&mut self) {
        self.reset = true;
    }

    /// Applies the movement given some delta time.
    pub fn update(
        &mut self,
        delta_ms: u128,
        mut body_part_query: Query<(&mut Transform, &BodyPart, With<BodyPart>)>,
    ) {
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

        let body = &mut body[0];

        if self.reset {
            self.reset = false;

            // move Arenito's body parts to their position relative to the origin
            body.translation = self.center;
            let r = body.rotation.inverse();
            body.rotate_around(Vec3::ZERO, r);

            // resets attributes
            self.center = Vec3::new(0.0, 0.5, 0.0);
            self.acc = Vec3::ZERO;
            self.vel = Vec3::ZERO;
            self.state = ArenitoState::STILL;
            self.look_angle = 0.0;

            return;
        }

        let delta: f32 = delta_ms as f32 / 1000.0;
        let fric = self.acc.normalize_or_zero() * -1.0 * FRIC_K;

        self.acc += fric; // ya está invertido
        self.vel = (self.acc * delta) + self.vel;

        if self.acc.length() < FRIC_K {
            self.vel = Vec3::ZERO;
            self.acc = Vec3::ZERO;
            self.state = ArenitoState::STILL;
        }

        let d = (self.vel * delta) + (0.5 * self.acc * delta * delta);
        let dl = d.length();

        if self.state == ArenitoState::FORWARD {
            self.center += d;
            body.translation += d;

            // wheel visual rotation
            for wheel in &mut left_wheels {
                wheel.rotate_local_z(-dl);
            }
            for wheel in &mut right_wheels {
                wheel.rotate_local_z(-dl);
            }
        } else {
            let theta = dl * self.state as isize as f32;
            self.look_angle = (self.look_angle + theta) % TAU;

            body.translation -= self.center;
            body.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-theta));
            body.translation += self.center;

            for wheel in &mut left_wheels {
                wheel.rotate_local_z(-theta);
            }
            for wheel in &mut right_wheels {
                wheel.rotate_local_z(theta);
            }
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
