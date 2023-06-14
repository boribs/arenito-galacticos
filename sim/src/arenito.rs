use bevy::prelude::*;

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
}

impl Arenito {
    pub fn new() -> Self {
        Arenito {
            center: Vec3::new(-3.0, 0.5, 0.0),
            vel: Vec3::new(0.2, 0.0, 0.1),
            acc: Vec3::ZERO,
            look_angle: 0.0,
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
        todo!("forward movement");
    }

    /// Sets Arenito in "rotation mode" - sets the rotation acceleration
    /// to the correct values.
    pub fn rotate(&mut self) {
        todo!("rotation (tank controls)");
    }

    /// Applies the movement given some delta time.
    pub fn update(
        &mut self,
        delta_ms: u128,
        mut body_part_query: Query<&mut Transform, With<BodyPart>>,
    ) {
        let delta: f32 = delta_ms as f32 / 1000.0;
        let d = self.vel * delta;

        for mut body_part in &mut body_part_query {
            body_part.translation += d;
        }

        self.center += d;

        //TODO: Update speed based on acceleration
        //TODO: Consider rotation update!
    }
}
