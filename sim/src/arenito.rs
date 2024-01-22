use crate::{
    cans::CanData,
    collision::WithDistanceCollision,
    sensor::{AISimMem, FromGyro, SimInstruction},
    static_shape::*,
};
use bevy::{
    prelude::*,
    render::{camera::RenderTarget, view::screenshot::ScreenshotManager},
    window::{Window, WindowRef, WindowResolution},
};
use bevy_obj::*;

const IMG_WIDTH: f32 = 512.0;
const IMG_HEIGHT: f32 = 512.0;

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
    pub enable_can_eating: bool,
}

impl Plugin for ArenitoPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ObjPlugin>() {
            app.add_plugins(ObjPlugin);
        }

        app.insert_resource(Arenito::new())
            .add_systems(Startup, arenito_spawner)
            .add_systems(Update, arenito_ai_mover);

        if self.enable_can_eating {
            app.add_systems(Update, eat_cans);
        }
    }
}

/// Spawns Arenito.
fn arenito_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut arenito: ResMut<Arenito>,
) {
    arenito.spawn(&mut commands, &mut meshes, &mut materials, &asset_server);
}

/// Reads user input and makes Arenito move.
fn arenito_mover(
    time: Res<Time>,
    mut arenito: ResMut<Arenito>,
    keyboard_input: Res<Input<KeyCode>>,
    mut arenito3d: Query<(&mut Transform, &Arenito3D, Entity)>,
) {
    todo!()
}

/// Gets movement instruction from AI and executes.
fn arenito_ai_mover(
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut arenito: ResMut<Arenito>,
    mut aisim: ResMut<AISimMem>,
    window: Query<Entity, With<ArenitoCamWindow>>,
) {
    if let Some(instr) = aisim.get_instruction() {
        todo!()
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

    arenito.draw_sphere(Color::WHITE, &mut gizmos);
}
/* --------------------------/Arenito Plugin---------------------------- */

/// Component used as an identifier for the different
/// body parts in 3D Arenito.
#[derive(Component, PartialEq)]
pub enum Arenito3D {
    Frame,
    LeftWheel,
    RightWheel,
    Brush,
}

#[derive(Component)]
pub struct ArenitoCamera;

#[derive(Component)]
pub struct ArenitoCamWindow;

#[derive(Clone, Copy, Debug)]
enum BaseInstruction {
    Forward,
    Left,
    Right,
}

#[derive(PartialEq)]
enum HandlerState {
    Waiting,
    Executing,
    Done,
}

/// Arenito's instructions are a combination of base instructions
/// (move forward, backwards, left, right) and a time stamp (how long
/// should that instruction be executed).
/// There are also combined instructions (move back, then right).
/// This struct keeps track of how long has an instruction been executed
/// and what the next ones are.
struct InstructionHandler {
    instructions: Vec<(BaseInstruction, f32)>,
    remaining_time: f32,
    state: HandlerState,
}

impl InstructionHandler {
    fn wait(&mut self) {
        self.state = HandlerState::Waiting;
    }

    fn execute(&mut self) {
        self.state = HandlerState::Executing;
    }

    fn done(&mut self) {
        println!("done");
        self.state = HandlerState::Done;
    }

    /// Sets the next instruction set.
    /// Converts SimInstruction to BaseInstructions.
    fn set(&mut self, instruction: SimInstruction) {
        println!("Setting {:?}", instruction);
        match instruction {
            SimInstruction::MoveForward => {
                self.instructions = vec![(BaseInstruction::Forward, 0.1)];
            }
            SimInstruction::MoveLeft => {
                self.instructions = vec![(BaseInstruction::Left, 0.05)];
            }
            SimInstruction::MoveRight => {
                self.instructions = vec![(BaseInstruction::Right, 0.05)];
            }
            other => panic!("Instruction {:?} not supported!", other),
        }

        self.remaining_time = self.instructions[0].1;
        self.state = HandlerState::Executing;
    }

    /// Returns current base instruction with its remaining execution time.
    fn current(&self) -> Option<(BaseInstruction, f32)> {
        if self.instructions.len() == 0 {
            None
        } else {
            Some((self.instructions[0].0, self.remaining_time))
        }
    }

    /// Removes current instruction and advances to the next one.
    /// Sets remaining time to instruction's total execution time.
    fn next(&mut self) {
        println!("Getting next");

        self.instructions.remove(0);

        if self.instructions.len() == 0 {
            self.done();
        } else {
            println!("next is: {:?}", self.instructions[0]);
            self.remaining_time = self.instructions[0].1;
        }
    }
}

impl Default for InstructionHandler {
    fn default() -> Self {
        InstructionHandler {
            instructions: Vec::with_capacity(2),
            remaining_time: 0.0,
            state: HandlerState::Waiting,
        }
    }
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
    // Maybe put cam data inside CameraArea -- rename it to CameraData
    pub cam_offset: Vec3, // cam pos relative to Arenito's center
    pub cam_area: CameraArea,
    brush_offset: Vec3, // brush pos relative to Arenito's center
    instruction_handler: InstructionHandler,
}

impl Arenito {
    const ACCEL_SPEED: f32 = 4.0;
    pub const MAX_VELOCITY: f32 = 3.0;
    const ROT_SPEED: f32 = 1.5;
    const BRUSH_SPEED: f32 = 10.0;
    pub const CENTER: Vec3 = Vec3 {
        x: 0.0,
        y: 0.2,
        z: 0.0,
    };

    /// Returns an empty, non-spawned Arenito.
    pub fn new() -> Self {
        Arenito {
            center: Self::CENTER,
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            rot: Vec3::ZERO,
            cam_offset: Vec3::new(0.75, 1.3, 0.0),
            cam_area: CameraArea::default(),
            brush_offset: Vec3::new(0.75, 0.4, 0.0),
            instruction_handler: InstructionHandler::default(),
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
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
    ) {
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

                let bt = Transform::from_xyz(
                    self.brush_offset.x,
                    self.brush_offset.y,
                    self.brush_offset.z,
                );
                // rotating brush!
                parent.spawn((
                    PbrBundle {
                        mesh: asset_server.load("models/cerdas.obj"),
                        material: materials.add(Color::VIOLET.into()),
                        transform: bt,
                        ..default()
                    },
                    Arenito3D::Brush,
                ));

                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.08, 0.08, 0.9).into()),
                    material: materials.add(Color::GRAY.into()),
                    transform: bt,
                    ..default()
                });

                // Arenito mounted camera
                let (x, y, z) = (self.cam_offset.x, self.cam_offset.y, self.cam_offset.z);
                self.cam_area.compute_area(self.cam_offset, Self::CENTER.y);

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

                // second window
                // needed to capture Arenito's camera view.
                let window = parent
                    .spawn((
                        Window {
                            title: "Arenito view".to_owned(),
                            visible: false,
                            resolution: WindowResolution::new(IMG_WIDTH, IMG_HEIGHT),
                            resizable: false,
                            ..default()
                        },
                        ArenitoCamWindow,
                    ))
                    .id();

                let mut t = Transform::from_xyz(x, y, z).looking_to(Vec3::X, Vec3::Y);
                t.rotate_z(self.cam_area.alpha + 0.001);
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
            });
    }

    /// Resets the state of Arenito.
    /// This includes despawning and spawning the models. It was easier than
    /// resetting everything to it's original state.
    pub fn reset(&mut self, arenito3d: &mut Query<(&mut Transform, &Arenito3D, Entity)>) {
        self.center = Self::CENTER;
        self.acc = Vec3::ZERO;
        self.vel = Vec3::ZERO;
        self.rot = Vec3::ZERO;

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
        let delta = delta_ms as f32 / 1000.0;
        let (pos, rot) = self.update_pos(delta);
        self.update_model(pos, rot, delta, arenito3d);
    }

    /// Calculates position difference after executing `instruction`.
    fn calculate_next_pos(&self, instruction: BaseInstruction, time: f32) -> (Vec3, Quat) {
        match instruction {
            BaseInstruction::Forward => (
                Vec3::from_gyro(&self.rot) * Self::MAX_VELOCITY * time,
                Quat::IDENTITY,
            ),
            BaseInstruction::Left => todo!(),
            BaseInstruction::Right => todo!(),
        }
    }

    /// Updates Arenito's position given some time in seconds (`delta`).
    /// This method is suposed to be called every frame, where delta
    /// is the time between this frame's render and the previous.
    fn update_pos(&mut self, delta: f32) -> (Vec3, Quat) {
        let mut pos = Vec3::ZERO;
        let mut rot = Quat::IDENTITY;
        let mut delta = delta;

        // get (current instruction, remaining execution time)
        // return if no current instruction
        // if delta > remaining execution time
        // calculate_next_pos with remaining execution time
        // delta = delta - remaining execution time
        // add next pos to final pos/rot diff
        // current = get next instruction

        // delta = min(delta, remaining execution time)
        // caulcate_next_pos with delta
        // remaining execution time -= delta

        if let Some((instr, rem_time)) = self.instruction_handler.current() {
            if delta > rem_time {
                println!("Less than remaining time.");
                let (npos, nrot) = self.calculate_next_pos(instr, rem_time);
                pos += npos;
                rot *= nrot;
                delta -= rem_time;
                self.instruction_handler.next();
            }

            match self.instruction_handler.current() {
                None => {}
                Some((instr, rem_time)) => {
                    let time = delta.min(rem_time);
                    println!("executing for {}s", time);
                    let (npos, nrot) = self.calculate_next_pos(instr, time);
                    pos += npos;
                    rot *= nrot;
                    self.instruction_handler.remaining_time -= time;
                }
            };
        }

        (pos, rot)
    }

    /// Updates Arenito's rendered model.
    fn update_model(
        &self,
        pos_diff: Vec3,
        rot_diff: Quat,
        delta: f32,
        mut arenito3d: Query<(&mut Transform, &Arenito3D, Entity)>,
    ) {
        // Saving different body parts to their own variable.
        // Each body part behaves differently.
        let mut body = Vec::<Mut<'_, Transform>>::with_capacity(1);
        let mut brush = Vec::<Mut<'_, Transform>>::with_capacity(1);
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
                Arenito3D::Brush => {
                    brush.push(body_part.0);
                }
            }
        }

        let body = &mut body[0];
        let brush = &mut brush[0];
        brush.rotate_local_z(-Self::BRUSH_SPEED * delta);

        body.translation += pos_diff;

        // todo!()
    }

    /// Prints the current stats of Arenito.
    pub fn log(&self) -> String {
        format!(
            "c: {} acc: {} vel: {} º: {}",
            self.center, self.acc, self.vel, self.rot
        )
    }
}

impl WithDistanceCollision for Arenito {
    fn get_pos(&self) -> Vec3 {
        let q = Quat::from_euler(EulerRot::XYZ, self.rot.x, -self.rot.y, self.rot.z);
        q.mul_vec3(self.brush_offset) + self.center
    }

    fn get_radius(&self) -> f32 {
        0.4
    }
}

/// Despawns cans when collided with Arenito
pub fn eat_cans(mut commands: Commands, arenito: Res<Arenito>, cans: Query<(&CanData, Entity)>) {
    for (can, ent) in cans.iter() {
        if arenito.collides_with_dist(can) {
            commands.entity(ent).despawn();
        }
    }
}

#[cfg(test)]
mod arenito_tests {}
