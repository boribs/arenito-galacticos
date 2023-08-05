use crate::arenito::*;
use crate::spatial_awareness::FromGyro;
use crate::wire::*;
use bevy::prelude::*;
use bevy_obj::*;

/// A plugin for adding Arenito (the 3D robot) to
/// the app. This is to help declutter `main.rs`.
///
/// This plugin adds:
/// - Arenito resource
/// - Arenito spawner startup system
/// - Arenito's wires startup system
/// - Arenito mover system
///
/// *It also requires that `ObjPlugin` is added.
pub struct ArenitoPlugin {
    pub show_wires: bool,
}

impl Plugin for ArenitoPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ObjPlugin>() {
            app.add_plugin(ObjPlugin);
        }

        // indicator wires
        if self.show_wires {
            app.add_startup_system(wire_spawner).add_system(wire_mover);
        }

        // resources
        app.insert_resource(Arenito::new());
        // startup systems
        app.add_startup_system(arenito_spawner);
        // systems
        app.add_system(arenito_mover);
    }
}

/// Indication Wire components, for querying them.
#[derive(Component)]
enum WireComponent {
    VELOCITY,
    ACCELERATOIN,
    ROTATION,
}

/// Adds Arenito's indicator wires to the scene.
fn wire_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    Wire::spawn_unique(
        Vec3::ZERO,
        Vec3::ZERO,
        [1.0, 1.0, 0.0],
        WireComponent::VELOCITY,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    Wire::spawn_unique(
        Vec3::ZERO,
        Vec3::ZERO,
        [1.0, 0.0, 0.0],
        WireComponent::ACCELERATOIN,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    Wire::spawn_unique(
        Vec3::ZERO,
        Vec3::ZERO,
        [0.0, 0.0, 1.0],
        WireComponent::ROTATION,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

/// Adds Arenito to the scene.
fn arenito_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    arenito: Res<Arenito>,
) {
    arenito.spawn(&mut commands, &mut materials, &asset_server);
}

/// Moves the wires that indicate direction, speed and acceleration.
fn wire_mover(
    arenito: ResMut<Arenito>,
    mut wire: Query<(&mut Wire, &Handle<Mesh>, &WireComponent)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut vel: Option<(Mut<'_, Wire>, &Handle<Mesh>, &WireComponent)> = None;
    let mut acc: Option<(Mut<'_, Wire>, &Handle<Mesh>, &WireComponent)> = None;
    let mut rot: Option<(Mut<'_, Wire>, &Handle<Mesh>, &WireComponent)> = None;

    for w in &mut wire {
        match w.2 {
            WireComponent::ACCELERATOIN => acc = Some(w),
            WireComponent::VELOCITY => vel = Some(w),
            WireComponent::ROTATION => rot = Some(w),
        }
    }

    let mut vel = vel.unwrap();
    let mut acc = acc.unwrap();
    let mut rot = rot.unwrap();

    let rup = Vec3::new(0.0, 1.6, 0.0);
    let vup = Vec3::new(0.0, 1.7, 0.0);
    let aup = Vec3::new(0.0, 1.8, 0.0);

    vel.0.set_start(arenito.center + vup);
    vel.0.set_end(arenito.center + arenito.vel + vup);
    vel.0.update(meshes.get_mut(vel.1).unwrap());

    acc.0.set_start(arenito.center + aup);
    acc.0.set_end(arenito.center + arenito.acc + aup);
    acc.0.update(meshes.get_mut(acc.1).unwrap());

    // let r = sensor::MPU6050::read_rot(&arenito);
    let r = arenito.rot;
    let rvec = Vec3::from_gyro(&r);
    rot.0.set_start(arenito.center + rup);
    rot.0.set_end(arenito.center + rvec + rup);
    rot.0.update(meshes.get_mut(rot.1).unwrap());
}

/// Reads user input and makes Arenito move.
fn arenito_mover(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arenito: ResMut<Arenito>,
    body_part_query: Query<(&mut Transform, &BodyPart, Entity)>,
) {
    if keyboard_input.pressed(KeyCode::W) {
        arenito.forward();
    } else if keyboard_input.pressed(KeyCode::A) {
        arenito.rotate(ArenitoState::LEFT);
    } else if keyboard_input.pressed(KeyCode::D) {
        arenito.rotate(ArenitoState::RIGHT);
    } else if keyboard_input.pressed(KeyCode::R) {
        arenito.reset(
            &mut commands,
            &asset_server,
            &mut materials,
            &body_part_query,
        );
    }

    arenito.update(time.delta().as_millis(), body_part_query);
    // println!("{}", arenito.log());
}
