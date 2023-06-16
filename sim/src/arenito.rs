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

#[derive(Resource)]
pub struct Arenito {
    center: Vec3,
    pub vel: Vec3,
    pub acc: Vec3,
    look_angle: f32, // on the y axis
    rotating: bool,
}

impl Arenito {
    pub fn new() -> Self {
        Arenito {
            center: Vec3::new(-3.0, 0.5, 0.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
            acc: Vec3::new(0.0, 0.0, 0.0),
            look_angle: 0.0,
            rotating: false,
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
        let (sin, cos) = self.look_angle.sin_cos();
        self.acc = Vec3::new(cos, 0.0, sin) * ACCEL_SPEED;
        self.rotating = false;
    }

    /// Sets Arenito in "rotation mode" - sets the rotation acceleration
    /// to the correct values.
    pub fn rotate(&mut self) {
        self.acc = Vec3::ONE * ROT_SPEED;
        self.rotating = true;
    }

    /// Applies the movement given some delta time.
    pub fn update(
        &mut self,
        delta_ms: u128,
        mut body_part_query: Query<&mut Transform, With<BodyPart>>,
    ) {
        let delta: f32 = delta_ms as f32 / 1000.0;

        let fric_nor = self.acc.normalize_or_zero() * -1.0;
        let fric = fric_nor * FRIC_K;

        self.acc += fric; // ya está invertido
        self.vel = (self.acc * delta) + self.vel;

        if self.acc.length() < FRIC_K {
            self.vel = Vec3::ZERO;
            self.acc = Vec3::ZERO;
            self.rotating = false;
        }

        let d = (self.vel * delta) + (0.5 * self.acc * delta * delta);
        println!(
            "v: {} a: {} º: {} - {}",
            self.vel,
            self.acc,
            self.look_angle,
            self.rotating
        );

        if !self.rotating {
            self.center += d;
            for mut body_part in &mut body_part_query {
                body_part.translation += d;
            }
        } else {
            let theta = d.length();
            self.look_angle = (self.look_angle + theta) % TAU;

            for mut body_part in &mut body_part_query {
                body_part.translation -= self.center;
                body_part.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-theta));
                body_part.translation += self.center;
            }
        }
    }
}
