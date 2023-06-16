use bevy::prelude::*;
use std::f32::consts::TAU;

const ACCEL_SPEED: f32 = 4.0;
const ROT_SPEED: f32 = 1.5;
const FRIC_K: f32 = 0.3;

#[derive(Component)]
pub struct BodyPart;

// #[derive(Component)]
// pub struct LeftWheel;
// #[derive(Component)]
// pub struct RightWheel;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ArenitoDirection {
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
    pub direction: ArenitoDirection,
    pub look_angle: f32, // on the y axis
    reset: bool
}

impl Arenito {
    pub fn new() -> Self {
        Arenito {
            center: Vec3::new(0.0, 0.5, 0.0),
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            look_angle: 0.0,
            direction: ArenitoDirection::STILL,
            reset: false,
        }
    }

    // Spawns Arenito into the scene
    pub fn spawn(
        &self,
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn((
            PbrBundle {
                mesh: asset_server.load("arenito.obj"),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(self.center.x, self.center.y, self.center.z),
                ..default()
            },
            BodyPart,
        ));

        let t = self.center + Vec3::new(0.5, 0.0, 0.85);
        commands.spawn((
            PbrBundle {
                mesh: asset_server.load("rueda.obj"),
                material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                transform: Transform::from_xyz(t.x, t.y, t.z),
                ..default()
            },
            BodyPart,
        ));

        let t = self.center + Vec3::new(-0.5, 0.0, 0.85);
        commands.spawn((
            PbrBundle {
                mesh: asset_server.load("rueda.obj"),
                material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                transform: Transform::from_xyz(t.x, t.y, t.z),
                ..default()
            },
            BodyPart,
        ));
        let t = self.center + Vec3::new(0.5, 0.0, -0.85);
        commands.spawn((
            PbrBundle {
                mesh: asset_server.load("rueda.obj"),
                material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                transform: Transform::from_xyz(t.x, t.y, t.z),
                ..default()
            },
            BodyPart,
        ));
        let t = self.center + Vec3::new(-0.5, 0.0, -0.85);
        commands.spawn((
            PbrBundle {
                mesh: asset_server.load("rueda.obj"),
                material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                transform: Transform::from_xyz(t.x, t.y, t.z),
                ..default()
            },
            BodyPart,
        ));
    }

    /// Sets the acceleration to "advance acceleration".
    pub fn forward(&mut self) {
        if self.direction != ArenitoDirection::STILL &&
           self.direction != ArenitoDirection::FORWARD {
            return;
        }

        let (sin, cos) = self.look_angle.sin_cos();
        self.acc = Vec3::new(cos, 0.0, sin) * ACCEL_SPEED;
        self.direction = ArenitoDirection::FORWARD;
    }

    /// Sets Arenito in "direction mode" - sets the direction acceleration
    /// to the correct values.
    pub fn rotate(&mut self, dir: ArenitoDirection) {
        if self.direction != ArenitoDirection::STILL &&
           self.direction != dir {
            return;
        }

        self.acc = Vec3::ONE * ROT_SPEED;
        self.direction = dir;
    }

    /// Applies the movement given some delta time.
    pub fn update(
        &mut self,
        delta_ms: u128,
        mut body_part_query: Query<&mut Transform, With<BodyPart>>,
    ) {
        if self.reset {
            self.reset = false;

            // move Arenito's body parts to their position relative to the origin
            for mut body_part in &mut body_part_query {
                body_part.translation -= self.center;
                let r = body_part.rotation.inverse();
                body_part.rotate_around(Vec3::ZERO, r);
            }

            // resets attributes
            self.center = Vec3::new(0.0, 0.5, 0.0);
            self.acc = Vec3::ZERO;
            self.vel = Vec3::ZERO;
            self.direction = ArenitoDirection::STILL;
            self.look_angle = 0.0;

            // Arenito's center is not the origin, so move every part to the center
            for mut body_part in &mut body_part_query {
                body_part.translation += self.center;
            }

            return;
        }

        let delta: f32 = delta_ms as f32 / 1000.0;
        let fric = self.acc.normalize_or_zero() * -1.0 * FRIC_K;

        self.acc += fric; // ya está invertido
        self.vel = (self.acc * delta) + self.vel;

        if self.acc.length() < FRIC_K {
            self.vel = Vec3::ZERO;
            self.acc = Vec3::ZERO;
            self.direction = ArenitoDirection::STILL;
        }

        let d = (self.vel * delta) + (0.5 * self.acc * delta * delta);

        if self.direction == ArenitoDirection::FORWARD {
            self.center += d;
            for mut body_part in &mut body_part_query {
                body_part.translation += d;
            }
        } else {
            let theta = d.length() * self.direction as isize as f32;
            self.look_angle = (self.look_angle + theta) % TAU;

            for mut body_part in &mut body_part_query {
                body_part.translation -= self.center;
                body_part.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-theta));
                body_part.translation += self.center;
            }
        }
    }

    /// Sets the state to reset on the next call to Arenito::update().
    pub fn reset(&mut self) {
        self.reset = true;
    }

    /// Prints the current stats of Arenito.
    pub fn log(&self) -> String {
        format!(
            "c: {} acc: {} vel: {} º: {} - {:?}",
            self.center,
            self.acc,
            self.vel,
            self.look_angle,
            self.direction
        )
    }
}
