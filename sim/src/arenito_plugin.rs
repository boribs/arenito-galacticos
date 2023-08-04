use crate::arenito::*;
use crate::spatial_awareness as sa;
use crate::spatial_awareness::FromGyro;
use crate::wire::*;
use bevy::prelude::*;

/// A plugin for adding Arenito (the 3D robot) to
/// the app. This is to help declutter `main.rs`.
///
/// This plugin adds:
/// - Arenito resource
/// - WirePath resource
/// - Calculated Movement resource
/// - Arenito spawner startup system
/// - Arenito wires startup system
/// - Arenito mover system
/// - Path finder system
///
/// *It also requires that `ObjPlugin` is added.
pub struct ArenitoPlugin;

impl Plugin for ArenitoPlugin {
    fn build(&self, app: &mut App) {
        // resources
        app.insert_resource(Arenito::new())
            .insert_resource(sa::CalculatedMovement::new())
            .insert_resource(WirePath::new([1.0, 1.0, 1.0]));
        // startup systems
        app.add_startup_system(arenito_spawner);
        // systems
        app.add_system(arenito_mover)
            .add_system(wire_mover)
            .add_system(sa::path_finder);
    }
}

#[derive(Component)]
enum WireComponent {
    VELOCITY,
    ACCELERATOIN,
    ROTATION,
}

fn arenito_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    arenito: Res<Arenito>,
    mut wirepath: ResMut<WirePath>,
) {
    arenito.spawn(&mut commands, &mut materials, &asset_server);
    wirepath.init_path(
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );

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
