use crate::{
    sensor::{AISimMem, SimInstruction},
    static_shape::*,
};
use bevy::{
    prelude::*,
    render::{camera::RenderTarget, view::screenshot::ScreenshotManager},
    window::{Window, WindowRef, WindowResolution},
};
use bevy_obj::*;
use std::f32::consts::TAU;

const FRIC_K: f32 = 0.5;

/* ----------------------------Arenito Plugin---------------------------- */

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
    pub img_width: f32,
    pub img_height: f32,
}

impl Plugin for ArenitoPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ObjPlugin>() {
            app.add_plugins(ObjPlugin);
        }

        app.insert_resource(Arenito::new(self.img_width, self.img_height))
            .add_systems(Startup, arenito_spawner)
            .add_systems(Update, (arenito_mover, arenito_ai_mover, draw_camera_area));
    }
}

/// Spawns Arenito.
fn arenito_spawner(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut arenito: ResMut<Arenito>,
) {
    arenito.spawn(&mut commands, &mut materials, &mut meshes, &asset_server);
}

/// Reads user input and makes Arenito move.
fn arenito_mover(
    time: Res<Time>,
    mut arenito: ResMut<Arenito>,
    keyboard_input: Res<Input<KeyCode>>,
    mut arenito3d: Query<(&mut Transform, &Arenito3D, Entity)>,
) {
    if keyboard_input.pressed(KeyCode::W) {
        arenito.forward();
    } else if keyboard_input.pressed(KeyCode::A) {
        arenito.rotate(ArenitoState::Left);
    } else if keyboard_input.pressed(KeyCode::D) {
        arenito.rotate(ArenitoState::Right);
    } else if keyboard_input.pressed(KeyCode::R) {
        arenito.reset(&mut arenito3d);
    }

    arenito.update(time.delta().as_millis(), arenito3d);
    // println!("{}", arenito.log());
}

/// Gets movement instruction from AI and executes.
fn arenito_ai_mover(
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut arenito: ResMut<Arenito>,
    mut aisim: ResMut<AISimMem>,
    window: Query<Entity, With<ArenitoCamWindow>>,
) {
    if let Some(instr) = aisim.get_instruction() {
        match instr {
            SimInstruction::Move(dir) => {
                match dir {
                    ArenitoState::Forward => arenito.forward(),
                    ArenitoState::Left => arenito.rotate(ArenitoState::Left),
                    ArenitoState::Right => arenito.rotate(ArenitoState::Right),
                    _ => {}
                };
                aisim.confirm_instruction();
            }
            SimInstruction::ScreenShot => {
                aisim.export_frame(&mut screenshot_manager, &window.single());
            }
        };
    }
}

fn draw_camera_area(arenito: Res<Arenito>, mut gizmos: Gizmos) {
    let mut points = arenito.cam_area.points.clone();
    let q = Quat::from_euler(EulerRot::XYZ, arenito.rot.x, -arenito.rot.y, arenito.rot.z);

    for i in 0..points.len() {
        points[i] = q.mul_vec3(points[i]) + Vec3::new(arenito.center.x, 0.0, arenito.center.z);
    }

    for i in 0..points.len() - 1 {
        gizmos.ray(points[i], points[i + 1] - points[i], Color::WHITE);
    }
    gizmos.ray(points[3], points[0] - points[3], Color::WHITE);
}
/* --------------------------/Arenito Plugin---------------------------- */

/// Component used as an identifier for the different
/// body parts in 3D Arenito.
#[derive(Component, PartialEq)]
pub enum Arenito3D {
    Frame,
    LeftWheel,
    RightWheel,
}

#[derive(Component)]
pub struct ArenitoCamera;

#[derive(Component)]
pub struct ArenitoCamWindow;

/// Describes Arenito's state.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ArenitoState {
    Left = -1,
    Right = 1,
    Forward,
    Still,
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
    pub rot: Vec3,
    pub state: ArenitoState,
    pub cam_offset: Vec3, // cam pos relative to Arenito's center
    pub cam_area: CameraArea,
    img_width: f32,
    img_height: f32,
}

impl Arenito {
    const ACCEL_SPEED: f32 = 4.0;
    const ROT_SPEED: f32 = 1.5;
    pub const MAX_VELOCITY: f32 = 3.0;
    pub const CENTER: Vec3 = Vec3 {
        x: 0.0,
        y: 0.2,
        z: 0.0,
    };

    /// Returns an empty, non-spawned Arenito.
    pub fn new(img_width: f32, img_height: f32) -> Self {
        Arenito {
            center: Self::CENTER,
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            rot: Vec3::ZERO,
            state: ArenitoState::Still,
            cam_offset: Vec3::new(0.75, 1.3 + Arenito::CENTER.y, 0.0),
            cam_area: CameraArea::default(),
            img_width,
            img_height,
        }
    }

    /// Spawns Arenito (body cube and wheels) into the scene.
    ///
    /// Arenito's model is a cube (parent) with four wheels (children).
    /// This is to preserve positional rotation (not having to manually
    /// rotate each wheel), facilitating significantly rotating the wheels
    /// on the z axis when moving forward or rotating.
    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        asset_server: &Res<AssetServer>,
    ) {
        self.cam_area.compute_area(self.cam_offset);

        // This is 3D Arenito!
        commands
            .spawn((
                PbrBundle {
                    mesh: asset_server.load("models/arenito.obj"),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(self.center.x, self.center.y, self.center.z),
                    ..default()
                },
                Arenito3D::Frame,
            ))
            .with_children(|parent| {
                let t = self.center + Vec3::new(0.5, -0.2, 0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("models/rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    Arenito3D::RightWheel,
                ));
                let t = self.center + Vec3::new(-0.5, -0.2, 0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("models/rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    Arenito3D::RightWheel,
                ));
                let t = self.center + Vec3::new(0.5, -0.2, -0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("models/rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    Arenito3D::LeftWheel,
                ));
                let t = self.center + Vec3::new(-0.5, -0.2, -0.85);
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("models/rueda.obj"),
                        material: materials.add(Color::rgb(0.8, 0.3, 0.6).into()),
                        transform: Transform::from_xyz(t.x, t.y, t.z),
                        ..default()
                    },
                    Arenito3D::LeftWheel,
                ));

                // Arenito mounted camera
                let (x, y, z) = (self.cam_offset.x, self.cam_offset.y - Self::CENTER.y, self.cam_offset.z);
                let mut t =
                    Transform::from_xyz(x, y, z)
                        .looking_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y);
                t.rotate_z(self.cam_area.alpha);

                // second window
                // needed to capture Arenito's camera view.
                let window = parent
                    .spawn((
                        Window {
                            title: "Arenito view".to_owned(),
                            visible: false,
                            resolution: WindowResolution::new(self.img_width, self.img_height),
                            resizable: false,
                            ..default()
                        },
                        ArenitoCamWindow,
                    ))
                    .id();

                parent.spawn((
                    Camera3dBundle {
                        camera: Camera {
                            target: RenderTarget::Window(WindowRef::Entity(window)),
                            ..default()
                        },
                        transform: t,
                        ..default()
                    },
                    ArenitoCamera,
                ));

                // Camera model
                parent.spawn(PbrBundle {
                    mesh: asset_server.load("models/camara.obj"),
                    material: materials.add(Color::BLACK.into()),
                    transform: Transform::from_xyz(x, y, z).with_rotation(Quat::from_euler(
                        EulerRot::ZYX,
                        self.cam_area.alpha,
                        0.0,
                        0.0,
                    )),
                    ..default()
                });

                // Area computation has to be done here, to spawn the mesh that
                // displays Arenito's FOV.
                parent.spawn(PbrBundle {
                    mesh: meshes.add(self.cam_area.get_mesh()),
                    material: materials.add(Color::WHITE.into()),
                    transform: Transform::from_xyz(0.0, -Self::CENTER.y + 0.01, 0.0),
                    ..default()
                });
            });
    }

    /// Sets the acceleration to "advance acceleration".
    pub fn forward(&mut self) {
        if self.state != ArenitoState::Still && self.state != ArenitoState::Forward {
            return;
        }

        let (sin, cos) = self.rot.y.sin_cos();
        self.acc = Vec3::new(cos, 0.0, sin) * Arenito::ACCEL_SPEED;
        self.state = ArenitoState::Forward;
    }

    /// Sets Arenito in "rotation mode" - sets the acceleration
    /// to the correct values.
    pub fn rotate(&mut self, dir: ArenitoState) {
        if self.state != ArenitoState::Still && self.state != dir {
            return;
        }

        self.acc = Vec3::ONE * Arenito::ROT_SPEED;
        self.state = dir;
    }

    /// Resets the state of Arenito.
    /// This includes despawning and spawning the models. It was easier than
    /// resetting everything to it's original state.
    pub fn reset(&mut self, arenito3d: &mut Query<(&mut Transform, &Arenito3D, Entity)>) {
        self.center = Self::CENTER;
        self.acc = Vec3::ZERO;
        self.vel = Vec3::ZERO;
        self.rot = Vec3::ZERO;
        self.state = ArenitoState::Still;

        for body_part in arenito3d {
            if *body_part.1 == Arenito3D::Frame {
                let mut transform = body_part.0;
                transform.translation = self.center;
                transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
                break;
            }
        }
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
    /// * `arenito3d` - Bevy's way of finding elements.
    pub fn update(
        &mut self,
        delta_ms: u128,
        arenito3d: Query<(&mut Transform, &Arenito3D, Entity)>,
    ) {
        let vec = self.update_pos(delta_ms);
        self.update_model(vec, arenito3d);
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
        if self.vel.length() > Arenito::MAX_VELOCITY {
            self.vel = self.vel.normalize() * Arenito::MAX_VELOCITY;
        }

        // If the force of friction is bigger than Arenito's forward acceleration
        // and the computation continues as is, Arenito will move backwards!
        // If Arenito is unable to overpower friction, then it should stop.
        if self.acc.length() < FRIC_K {
            self.vel = Vec3::ZERO;
            self.acc = Vec3::ZERO;
            self.state = ArenitoState::Still;
        }

        // Highschool physics: Distance = v_0 * t + (0.5 * a * t^2)
        // This is also valid when velocity (v_0) and acceleratoin (a)
        // are both vectors.
        let d = (self.vel * delta) + (0.5 * self.acc * delta * delta);
        let dl = d.length();

        if self.state == ArenitoState::Forward {
            self.center += d;

            return (d, dl); // TODO: return in a more rustesque way
        } else {
            let theta = dl * self.state as isize as f32;
            self.rot.y = (self.rot.y + theta) % TAU;

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
        mut arenito3d: Query<(&mut Transform, &Arenito3D, Entity)>,
    ) {
        // Saving different body parts to their own variable.
        // Each body part behaves differently.
        let mut body = Vec::<Mut<'_, Transform>>::with_capacity(1);
        let mut left_wheels = Vec::<Mut<'_, Transform>>::with_capacity(2);
        let mut right_wheels = Vec::<Mut<'_, Transform>>::with_capacity(2);

        for body_part in &mut arenito3d {
            match body_part.1 {
                Arenito3D::LeftWheel => {
                    left_wheels.push(body_part.0);
                }
                Arenito3D::RightWheel => {
                    right_wheels.push(body_part.0);
                }
                Arenito3D::Frame => {
                    body.push(body_part.0);
                }
            }
        }

        // Since body is only one element, shadow it out of the vector!
        let body = &mut body[0];
        let (d, l) = vec;

        match self.state {
            ArenitoState::Forward => {
                body.translation += d;

                for wheel in &mut left_wheels {
                    wheel.rotate_local_z(-l);
                }
                for wheel in &mut right_wheels {
                    wheel.rotate_local_z(-l);
                }
            }
            ArenitoState::Right | ArenitoState::Left => {
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
            self.center, self.acc, self.vel, self.rot, self.state
        )
    }
}

#[cfg(test)]
mod arenito_tests {
    // Test nomenclature:
    // <surface_type>_<initial_conditions>_<other>
    //
    // For example:
    // irregular_terrain_absolute_rest_arenito_inclines_to_left_on_right_hill

    use super::*;
    use rand::{prelude::thread_rng, Rng};
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    const F32_DIFF: f32 = 0.001;

    impl Arenito {
        /// Method for rapid object initialization, where camera output
        /// data is not needed.
        pub fn test() -> Self {
            Self::new(0.0, 0.0)
        }

        /// Initializes Arenito with some velocity and acceleration.
        fn vel_acc(vel: Vec3, acc: Vec3, cen: Vec3) -> Self {
            let mut arenito = Arenito::test();
            arenito.vel = vel;
            arenito.acc = acc;
            arenito.center = cen;
            arenito.state = ArenitoState::Forward;

            arenito
        }
    }

    /// Compares vectors' values.
    /// Considers "equal" values within a difference of `F32_DIFF`.
    fn cmp_vec(a: Vec3, b: Vec3) {
        assert!((a.x - b.x).abs() < F32_DIFF, "x: {} != {}", a.x, b.x);
        assert!((a.y - b.y).abs() < F32_DIFF, "y: {} != {}", a.y, b.y);
        assert!((a.z - b.z).abs() < F32_DIFF, "z: {} != {}", a.z, b.z);
    }

    /// Compares arenito's values with the provided ones.
    fn cmp_arenito(arenito: &Arenito, vel: &Vec3, acc: &Vec3, cen: &Vec3) {
        cmp_vec(arenito.vel, *vel);
        cmp_vec(arenito.acc, *acc);
        cmp_vec(arenito.center, *cen);
    }

    #[test]
    fn flat_surface_arenito_doesnt_move_with_0_acceleration_and_velocity() {
        let mut arenito = Arenito::test();

        // look angle really doesn't matter, but I guess it's
        // useful to make a point?
        for angle in [0.0, -1.31, 4.32, 6.16, -2.54] {
            arenito.rot.y = angle;
            arenito.update_pos(16);

            cmp_vec(arenito.vel, Vec3::ZERO);
            cmp_vec(arenito.acc, Vec3::ZERO);
            cmp_vec(arenito.center, Arenito::CENTER);
        }
    }

    // ------------------------------------------------------------
    // The following tests are to test Arenito's movement a single
    // frame forward, from absolute rest on a flat surface.
    // all of them assume FRIC_K = 0.5!!!

    #[test]
    fn flat_surface_absolute_rest_positive_x() {
        let mut arenito = Arenito::test();
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.056, 0.0, 0.0);
        let expected_acc = Vec3::new(3.5, 0.0, 0.0);
        let expected_center = Vec3::new(0.001344, Arenito::CENTER.y, 0.0);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_positive_xz() {
        let mut arenito = Arenito::test();
        arenito.rot.y = FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.03959, 0.0, 0.03959);
        let expected_acc = Vec3::new(2.47487, 0.0, 2.47487);
        let expected_center = Vec3::new(0.00095, Arenito::CENTER.y, 0.00095);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_positive_z() {
        let mut arenito = Arenito::test();
        arenito.rot.y = FRAC_PI_2;
        arenito.forward();
        arenito.update_pos(16);

        // most zeros aren't actually zero, but very close
        let expected_vel = Vec3::new(0.0, 0.0, 0.056);
        let expected_acc = Vec3::new(0.0, 0.0, 3.5);
        let expected_center = Vec3::new(0.0, Arenito::CENTER.y, 0.001344);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_negative_x_positive_z() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 3.0 * FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.03959, 0.0, 0.03959);
        let expected_acc = Vec3::new(-2.47487, 0.0, 2.47487);
        let expected_center = Vec3::new(-0.0009, Arenito::CENTER.y, 0.0009);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_negative_x() {
        let mut arenito = Arenito::test();
        arenito.rot.y = PI;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.056, 0.0, 0.0);
        let expected_acc = Vec3::new(-3.5, 0.0, 0.0);
        let expected_center = Vec3::new(-0.001344, Arenito::CENTER.y, 0.0);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_negative_x_negative_z() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 5.0 * FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.03959, 0.0, -0.03959);
        let expected_acc = Vec3::new(-2.47487, 0.0, -2.47487);
        let expected_center = Vec3::new(-0.0009, Arenito::CENTER.y, -0.0009);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_negative_z() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 3.0 * FRAC_PI_2;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.0, 0.0, -0.056);
        let expected_acc = Vec3::new(0.0, 0.0, -3.5);
        let expected_center = Vec3::new(0.0, Arenito::CENTER.y, -0.001344);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_negative_z_positive_x() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 7.0 * FRAC_PI_4;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.03959, 0.0, -0.03959);
        let expected_acc = Vec3::new(2.47487, 0.0, -2.47487);
        let expected_center = Vec3::new(0.00095, Arenito::CENTER.y, -0.00095);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_1() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 0.1234;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.05557, 0.0, 0.00689);
        let expected_acc = Vec3::new(3.47338, 0.0, 0.43080);
        let expected_center = Vec3::new(0.0013, Arenito::CENTER.y, 0.00016);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_2() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 0.38;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.05200, 0.0, 0.020771);
        let expected_acc = Vec3::new(3.25032, 0.0, 1.29822);
        let expected_center = Vec3::new(0.00124, Arenito::CENTER.y, 0.00049);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_3() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 4.7551;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.00239, 0.0, -0.055948);
        let expected_acc = Vec3::new(0.14944, 0.0, -3.49680);
        let expected_center = Vec3::new(0.0, Arenito::CENTER.y, -0.0013);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_4() {
        let mut arenito = Arenito::test();
        arenito.rot.y = -6.1362;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.0553, 0.0, 0.008197);
        let expected_acc = Vec3::new(3.46229, 0.0, 0.51233);
        let expected_center = Vec3::new(0.00132, Arenito::CENTER.y, 0.00019);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_5() {
        let mut arenito = Arenito::test();
        arenito.rot.y = -0.713244;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.042349, 0.0, -0.03664);
        let expected_acc = Vec3::new(2.6468, 0.0, -2.29001);
        let expected_center = Vec3::new(0.001016, Arenito::CENTER.y, -0.00087);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_6() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 3.70245;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.04742, 0.0, -0.02978);
        let expected_acc = Vec3::new(-2.9637, 0.0, -1.8617);
        let expected_center = Vec3::new(-0.00113, Arenito::CENTER.y, -0.00071);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_7() {
        let mut arenito = Arenito::test();
        arenito.rot.y = -1.4037;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.00930, 0.0, -0.05522);
        let expected_acc = Vec3::new(0.58178, 0.0, -3.45130);
        let expected_center = Vec3::new(0.00022, Arenito::CENTER.y, -0.00132);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_8() {
        let mut arenito = Arenito::test();
        arenito.rot.y = -1.4037;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.00930, 0.0, -0.05522);
        let expected_acc = Vec3::new(0.58178, 0.0, -3.45130);
        let expected_center = Vec3::new(0.00022, Arenito::CENTER.y, -0.00132);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_9() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 1.65394;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.00465, 0.0, 0.055806);
        let expected_acc = Vec3::new(-0.29068, 0.0, 3.487908);
        let expected_center = Vec3::new(-0.00011, Arenito::CENTER.y, 0.001339);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_absolute_rest_random_angle_10() {
        let mut arenito = Arenito::test();
        arenito.rot.y = 0.52525;
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.04845, 0.0, 0.02808);
        let expected_acc = Vec3::new(3.02817, 0.0, 1.75502);
        let expected_center = Vec3::new(0.00116, Arenito::CENTER.y, 0.00067);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    // ------------------------------------------------------------
    // The following tests are to test Arenito's movement a single
    // frame forward, already moving and accelerating, that is, the user
    // is pressing the `forward` button, on a flat surface.
    // These assume a `constant time between frames` of 16 ms.

    #[test]
    fn flat_surface_accelerating_positive_x() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.2332, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Arenito::CENTER,
        );
        arenito.forward();
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.2892, 0.0, 0.0);
        let expected_acc = Vec3::new(3.5, 0.0, 0.0);
        let expected_center = Vec3::new(0.00507, Arenito::CENTER.y, 0.0);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_positive_xz() {
        // no initial look angle since direction is in velocity.
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.84852, 0.0, 0.84852),
            Vec3::new(2.82842, 0.0, 2.82842),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        // println!("{}", arenito.acc);

        let expected_vel = Vec3::new(0.88812, 0.0, 0.88812);
        let expected_acc = Vec3::new(2.47487, 0.0, 2.47487);
        let expected_center = Vec3::new(0.01452, Arenito::CENTER.y, 0.01452);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_positive_z() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.0, 0.0, 1.05),
            Vec3::new(0.0, 0.0, 4.00),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.0, 0.0, 1.106);
        let expected_acc = Vec3::new(0.0, 0.0, 3.5);
        let expected_center = Vec3::new(0.0, Arenito::CENTER.y, 0.018144);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_negative_x_positive_z() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-0.86974, 0.0, 0.86974),
            Vec3::new(-2.82842, 0.0, 2.82842),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.90933, 0.0, 0.90933);
        let expected_acc = Vec3::new(-2.47487, 0.0, 2.47487);
        let expected_center = Vec3::new(-0.01486, Arenito::CENTER.y, 0.01486);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_negative_x() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-1.42583, 0.0, 0.0),
            Vec3::new(-4.0, 0.0, 0.0),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-1.48183, 0.0, 0.0);
        let expected_acc = Vec3::new(-3.5, 0.0, 0.0);
        let expected_center = Vec3::new(-0.02415728, Arenito::CENTER.y, 0.0);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_negative_x_negative_z() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-1.00821, 0.0, -1.00821),
            Vec3::new(-2.82842, 0.0, -2.82842),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-1.04781, 0.0, -1.04781);
        let expected_acc = Vec3::new(-2.47487, 0.0, -2.47487);
        let expected_center = Vec3::new(-0.01708, Arenito::CENTER.y, -0.01708);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_accelerating_rest_negative_z() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.0, 0.0, -1.25),
            Vec3::new(0.0, 0.0, -4.0),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.0, 0.0, -1.306);
        let expected_acc = Vec3::new(0.0, 0.0, -3.5);
        let expected_center = Vec3::new(0.0, Arenito::CENTER.y, -0.02134);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_negative_z_positive_x() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.88388, 0.0, -0.88388),
            Vec3::new(2.82842, 0.0, -2.82842),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.923481, 0.0, -0.92348);
        let expected_acc = Vec3::new(2.47487, 0.0, -2.47487);
        let expected_center = Vec3::new(0.015092, Arenito::CENTER.y, -0.01509);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_1() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(1.61179, 0.0, -0.013083),
            Vec3::new(3.99986, 0.0, -0.032467),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(1.66779, 0.0, -0.01353);
        let expected_acc = Vec3::new(3.499884, 0.0, -0.02840);
        let expected_center = Vec3::new(0.027132, Arenito::CENTER.y, -0.00022);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_2() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.71650, 0.00000, 0.73864),
            Vec3::new(2.78507, 0.00000, 2.87113),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.75549, 0.00000, 0.77884);
        let expected_acc = Vec3::new(2.43693, 0.00000, 2.51224);
        let expected_center = Vec3::new(0.01240, Arenito::CENTER.y, 0.01278);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_3() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-0.22446, 0.00000, -1.28566),
            Vec3::new(-0.68794, 0.00000, -3.94040),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.23409, 0.00000, -1.34082);
        let expected_acc = Vec3::new(-0.60194, 0.00000, -3.44785);
        let expected_center = Vec3::new(-0.00382, Arenito::CENTER.y, -0.02189);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_4() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-1.36747, 0.00000, 1.35444),
            Vec3::new(-2.84194, 0.00000, 2.81485),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-1.40726, 0.00000, 1.39384);
        let expected_acc = Vec3::new(-2.48670, 0.00000, 2.46299);
        let expected_center = Vec3::new(-0.02283, Arenito::CENTER.y, 0.02262);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_5() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-0.24098, 0.00000, -1.37428),
            Vec3::new(-0.69086, 0.00000, -3.93989),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.25065, 0.00000, -1.42944);
        let expected_acc = Vec3::new(-0.60450, 0.00000, -3.44740);
        let expected_center = Vec3::new(-0.00409, Arenito::CENTER.y, -0.02331);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_6() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.89284, 0.00000, 0.62933),
            Vec3::new(3.26943, 0.00000, 2.30452),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.93861, 0.00000, 0.66160);
        let expected_acc = Vec3::new(2.86075, 0.00000, 2.01646);
        let expected_center = Vec3::new(0.01538, Arenito::CENTER.y, 0.01084);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_7() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.70439, 0.00000, 1.62769),
            Vec3::new(1.58864, 0.00000, 3.67100),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(0.72663, 0.00000, 1.67909);
        let expected_acc = Vec3::new(1.39006, 0.00000, 3.21212);
        let expected_center = Vec3::new(0.01180, Arenito::CENTER.y, 0.02728);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_8() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-1.02897, 0.00000, -1.60829),
            Vec3::new(-2.15571, 0.00000, -3.36941),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-1.05915, 0.00000, -1.65546);
        let expected_acc = Vec3::new(-1.88625, 0.00000, -2.94823);
        let expected_center = Vec3::new(-0.01719, Arenito::CENTER.y, -0.02686);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_9() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-0.93676, 0.00000, -1.36294),
            Vec3::new(-2.26568, 0.00000, -3.29646),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.96848, 0.00000, -1.40909);
        let expected_acc = Vec3::new(-1.98247, 0.00000, -2.88441);
        let expected_center = Vec3::new(-0.01575, Arenito::CENTER.y, -0.02291);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    #[test]
    fn flat_surface_accelerating_random_angle_10() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(-0.48455, 0.00000, 1.38236),
            Vec3::new(-1.32316, 0.00000, 3.77482),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        let expected_vel = Vec3::new(-0.50307, 0.00000, 1.43521);
        let expected_acc = Vec3::new(-1.15777, 0.00000, 3.30297);
        let expected_center = Vec3::new(-0.00820, Arenito::CENTER.y, 0.02339);

        cmp_arenito(&arenito, &expected_vel, &expected_acc, &expected_center);
    }

    // ------------------------------------------------------------
    // I've now deemed unnecessary to test movement on every direction
    // since the tests above show that the robot does pretty much
    // how I'd expect it to do on basically every direction.
    // The following tests are for general movement behaviour:
    // - stopping
    // - limiting velocity

    #[test]
    fn flat_surface_decelerating_positive_x() {
        // Arenito should stop moving if it's acceleration is
        // less than that of friction.
        let mut arenito = Arenito::vel_acc(
            Vec3::new(1.10000, 0.00000, 0.00000),
            Vec3::new(0.47800, 0.00000, 0.00000),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        cmp_vec(arenito.vel, Vec3::ZERO);
        cmp_vec(arenito.acc, Vec3::ZERO);
        assert!(arenito.state == ArenitoState::Still);
    }

    #[test]
    fn flat_surface_decelerating_negative_z() {
        let mut arenito = Arenito::vel_acc(
            Vec3::new(0.0, 0.0, -1.10000),
            Vec3::new(0.0, 0.0, -0.47800),
            Arenito::CENTER,
        );
        arenito.update_pos(16);

        cmp_vec(arenito.vel, Vec3::ZERO);
        cmp_vec(arenito.acc, Vec3::ZERO);
        assert!(arenito.state == ArenitoState::Still);
    }

    #[test]
    fn flat_surface_decelerating_random_look_angle() {
        let mut rng = thread_rng();

        for _ in 0..100 {
            let (sin, cos) = rng.gen_range(-TAU..TAU).sin_cos();
            let mut arenito = Arenito::vel_acc(
                // whatever the direction may be, if acceleration
                // is less than friction, Arenito should stop.
                Vec3::new(cos, 0.0, sin) * rng.gen_range(0.0..FRIC_K),
                Vec3::new(cos, 0.0, sin) * rng.gen_range(0.0..FRIC_K),
                Arenito::CENTER,
            );
            arenito.update_pos(16);

            cmp_vec(arenito.vel, Vec3::ZERO);
            cmp_vec(arenito.acc, Vec3::ZERO);
            assert!(arenito.state == ArenitoState::Still);
        }
    }

    #[test]
    fn flat_surface_arenito_reaches_max_vel() {
        // sometimes the length of arenito's velocity will be something
        // like 3.00000000001, which is really close to Arenito's current
        // max velocity, but not quite it, which makes this test fail.
        let max_vel = Arenito::MAX_VELOCITY + 0.00001;

        let mut rng = thread_rng();
        for _ in 0..100 {
            let (sin, cos) = rng.gen_range(-TAU..TAU).sin_cos();
            let mut arenito = Arenito::vel_acc(
                Vec3::new(cos, 0.0, sin) * rng.gen_range(3.0..10.0),
                Vec3::ZERO,
                Arenito::CENTER,
            );
            arenito.forward();
            arenito.update_pos(16);

            assert!(arenito.vel.length() <= max_vel);
        }
    }
}
