use crate::{
    cans::CanData,
    collision::{GlobalTransform, WithDistanceCollision},
    sensor::{AISimMem, ProximitySensor, SimInstruction},
    static_shape::*,
};
use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        view::{screenshot::ScreenshotManager, RenderLayers},
    },
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

        app.add_systems(Startup, (arenito_spawner, gizmo_config))
            .add_systems(
                Update,
                (
                    arenito_ai_mover,
                    draw_camera_area,
                    keyboard_control,
                    scan_distance,
                ),
            );

        if self.enable_can_eating {
            app.add_systems(Update, eat_cans);
        }
    }
}

#[derive(Component)]
struct ControlText;

/// Spawns Arenito.
fn arenito_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut arenito = Arenito::new();
    arenito.spawn(&mut commands, &mut meshes, &mut materials, &asset_server);

    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(" Mode: ", style.clone()),
            TextSection::new(format!("{:?}", arenito.control_mode), style),
        ]),
        ControlText,
    ));
}

fn gizmo_config(mut config: ResMut<GizmoConfig>) {
    config.render_layers = RenderLayers::layer(1);
}

/// Reads user input and makes Arenito move.
fn keyboard_control(
    mut arenito: Query<&mut Arenito>,
    keyboard_input: Res<Input<KeyCode>>,
    mut text: Query<&mut Text, With<ControlText>>,
    mut arenito_frame: Query<&mut Transform, With<ArenitoCompFrame>>,
) {
    let mut arenito = arenito.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        arenito.control_mode = match arenito.control_mode {
            ControlMode::AI => ControlMode::Manual,
            ControlMode::Manual => ControlMode::AI,
        };

        let mut text = text.single_mut();
        text.sections[1].value = format!("{:?}", arenito.control_mode)
    } else if keyboard_input.just_pressed(KeyCode::R) {
        arenito.reset(&mut arenito_frame.single_mut());
    }

    if arenito.control_mode == ControlMode::Manual && arenito.instruction_handler.available() {
        if keyboard_input.pressed(KeyCode::W) {
            arenito.instruction_handler.set(SimInstruction::MoveForward);
        } else if keyboard_input.pressed(KeyCode::S) {
            arenito.instruction_handler.set(SimInstruction::MoveBack);
        } else if keyboard_input.pressed(KeyCode::A) {
            arenito.instruction_handler.set(SimInstruction::MoveLeft);
        } else if keyboard_input.pressed(KeyCode::D) {
            arenito.instruction_handler.set(SimInstruction::MoveRight);
        }
    }
}

/// Gets movement instruction from AI and executes.
fn arenito_ai_mover(
    time: Res<Time>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut arenito: Query<&mut Arenito>,
    mut aisim: ResMut<AISimMem>,
    window: Query<Entity, With<ArenitoCamWindow>>,
    arenito_body: ParamSet<(
        Query<&mut Transform, With<ArenitoCompFrame>>,
        Query<&mut Transform, With<ArenitoCompBrush>>,
        Query<&mut Transform, With<ArenitoCompLeftWheel>>,
        Query<&mut Transform, With<ArenitoCompRightWheel>>,
    )>,
) {
    let mut arenito = arenito.single_mut();

    if arenito.control_mode == ControlMode::AI {
        match arenito.instruction_handler.state {
            HandlerState::Done => {
                aisim.confirm_instruction();
                arenito.instruction_handler.wait();
            }
            HandlerState::Waiting => {
                if let Some(instr) = aisim.get_instruction() {
                    if instr == SimInstruction::ScreenShot {
                        aisim.export_frame(&mut screenshot_manager, &window.single());
                        aisim.confirm_instruction();
                    } else {
                        arenito.instruction_handler.set(instr);
                        arenito.instruction_handler.execute();
                    }
                }
            }
            _ => {}
        }
    }

    arenito.update(time.delta().as_millis(), arenito_body);
}

fn draw_camera_area(arenito: Query<(&Arenito, &Transform)>, mut gizmos: Gizmos) {
    let (arenito, transform) = arenito.single();
    let mut points = arenito.cam_area.points.clone();
    let rot = transform.rotation;
    let pos = transform.translation;

    for i in 0..points.len() {
        points[i] = rot.mul_vec3(points[i]) + Vec3::new(pos.x, 0.0, pos.z);
    }

    for i in 0..points.len() - 1 {
        gizmos.line(points[i], points[i + 1], Color::WHITE);
    }
    gizmos.line(points[3], points[0], Color::WHITE);

    // This should not be here
    arenito.draw_sphere(transform, Color::WHITE, &mut gizmos);
}
/* --------------------------/Arenito Plugin---------------------------- */

#[derive(Clone, Copy, Debug)]
enum BaseInstruction {
    Back,
    Forward,
    Left,
    Right,
}

#[derive(PartialEq, Clone)]
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
#[derive(Clone)]
struct InstructionHandler {
    instructions: Vec<(BaseInstruction, f32)>,
    remaining_time: f32,
    state: HandlerState,
}

impl InstructionHandler {
    /// For manual mode. Indicates if InstructionHandler is done executing.
    fn available(&self) -> bool {
        self.state != HandlerState::Executing
    }

    fn wait(&mut self) {
        self.state = HandlerState::Waiting;
    }

    fn execute(&mut self) {
        self.state = HandlerState::Executing;
    }

    fn done(&mut self) {
        self.state = HandlerState::Done;
    }

    /// Sets the next instruction set.
    /// Converts SimInstruction to BaseInstructions.
    fn set(&mut self, instruction: SimInstruction) {
        // println!("Setting {:?}", instruction);
        match instruction {
            SimInstruction::MoveBack => {
                self.instructions = vec![(BaseInstruction::Back, 0.1)];
            }
            SimInstruction::MoveForward => {
                self.instructions = vec![(BaseInstruction::Forward, 0.1)];
            }
            SimInstruction::MoveLeft => {
                self.instructions = vec![(BaseInstruction::Left, 0.05)];
            }
            SimInstruction::MoveRight => {
                self.instructions = vec![(BaseInstruction::Right, 0.05)];
            }
            SimInstruction::MoveLongRight => {
                self.instructions = vec![(BaseInstruction::Right, 0.6)];
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
        // println!("Getting next");

        self.instructions.remove(0);

        if self.instructions.len() == 0 {
            self.done();
        } else {
            // println!("next is: {:?}", self.instructions[0]);
            self.remaining_time = self.instructions[0].1;
        }
    }

    /// Resets the instruction handler.
    fn reset(&mut self) {
        self.instructions.clear();
        self.remaining_time = 0.0;
        self.state = HandlerState::Done;
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

#[derive(Debug, PartialEq, Clone)]
enum ControlMode {
    Manual,
    AI,
}

#[derive(Component)]
pub struct ArenitoCompFrame;

#[derive(Component)]
pub struct ArenitoCompLeftWheel;

#[derive(Component)]
pub struct ArenitoCompRightWheel;

#[derive(Component)]
pub struct ArenitoCompBrush;

#[derive(Component)]
pub struct ArenitoCamera;

#[derive(Component)]
pub struct ArenitoCamWindow;

/// Arenito is the main component of this simulation.
///
/// It's responsible of both visual and "logical" updates of position,
/// velocity, acceleration and rotation.
/// Those attributes will be important when simulating the sensors.
#[derive(Component, Clone)]
pub struct Arenito {
    pub vel: Vec3,
    pub acc: Vec3,
    // Maybe put cam data inside CameraArea -- rename it to CameraData
    pub cam_offset: Vec3, // cam pos relative to Arenito's center
    pub cam_area: CameraArea,
    brush_offset: Vec3, // brush pos relative to Arenito's center
    instruction_handler: InstructionHandler,
    control_mode: ControlMode,
    proximity_sensor_offsets: Vec<Transform>,
}

impl Arenito {
    pub const VELOCITY: f32 = 1.5;
    const BRUSH_SPEED: f32 = 10.0;
    pub const CENTER: Vec3 = Vec3 {
        x: 0.0,
        y: 0.2,
        z: 0.0,
    };

    /// Returns an empty, non-spawned Arenito.
    pub fn new() -> Self {
        Arenito {
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            cam_offset: Vec3::new(0.75, 1.3, 0.0),
            cam_area: CameraArea::default(),
            brush_offset: Vec3::new(0.75, 0.4, 0.0),
            instruction_handler: InstructionHandler::default(),
            control_mode: ControlMode::AI,
            proximity_sensor_offsets: vec![Transform::from_xyz(0.74, 0.1, 0.0)],
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
        self.cam_area.compute_area(self.cam_offset, Self::CENTER.y);

        commands
            .spawn((
                PbrBundle {
                    mesh: asset_server.load("models/arenito.obj"),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_xyz(Self::CENTER.x, Self::CENTER.y, Self::CENTER.z),
                    ..default()
                },
                ArenitoCompFrame,
                self.clone(),
            ))
            .with_children(|parent| {
                const WOX: f32 = 0.5;
                const WOY: f32 = -0.2;
                const WOZ: f32 = 0.85;

                let rwheel_offsets = [Vec3::new(WOX, -WOY, WOZ), Vec3::new(-WOX, -WOY, WOZ)];
                let lwheel_offsets = [Vec3::new(WOX, -WOY, -WOZ), Vec3::new(-WOX, -WOY, -WOZ)];

                let wheel_mesh = asset_server.load("models/rueda.obj");
                let wheel_material = materials.add(Color::rgb(0.8, 0.3, 0.6).into());

                for wheel_offset in rwheel_offsets.iter() {
                    let t = Self::CENTER + *wheel_offset;

                    parent.spawn((
                        PbrBundle {
                            mesh: wheel_mesh.clone(),
                            material: wheel_material.clone(),
                            transform: Transform::from_xyz(t.x, t.y, t.z),
                            ..default()
                        },
                        ArenitoCompRightWheel,
                    ));
                }

                for wheel_offset in lwheel_offsets.iter() {
                    let t = Self::CENTER + *wheel_offset;

                    parent.spawn((
                        PbrBundle {
                            mesh: wheel_mesh.clone(),
                            material: wheel_material.clone(),
                            transform: Transform::from_xyz(t.x, t.y, t.z),
                            ..default()
                        },
                        ArenitoCompLeftWheel,
                    ));
                }

                for prox_offset in self.proximity_sensor_offsets.iter() {
                    parent.spawn((
                        PbrBundle {
                            transform: *prox_offset,
                            ..default()
                        },
                        ProximitySensor::default(),
                    ));
                }

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
                    ArenitoCompBrush,
                ));

                parent.spawn(PbrBundle {
                    mesh: meshes.add(shape::Box::new(0.08, 0.08, 0.9).into()),
                    material: materials.add(Color::GRAY.into()),
                    transform: bt,
                    ..default()
                });

                // Arenito mounted camera
                let (x, y, z) = (self.cam_offset.x, self.cam_offset.y, self.cam_offset.z);

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
    pub fn reset(&mut self, arenito_frame: &mut Transform) {
        self.acc = Vec3::ZERO;
        self.vel = Vec3::ZERO;
        self.instruction_handler.reset();

        arenito_frame.translation = Self::CENTER;
        arenito_frame.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
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
        mut arenito_body: ParamSet<(
            Query<&mut Transform, With<ArenitoCompFrame>>,
            Query<&mut Transform, With<ArenitoCompBrush>>,
            Query<&mut Transform, With<ArenitoCompLeftWheel>>,
            Query<&mut Transform, With<ArenitoCompRightWheel>>,
        )>,
    ) {
        let delta = delta_ms as f32 / 1000.0;
        let (pos, rot) = self.update_pos(delta, arenito_body.p0().single());

        self.update_model(pos, rot, delta, arenito_body);
    }

    /// Calculates position difference after executing `instruction`.
    fn calculate_next_pos(
        &self,
        transform: &Transform,
        instruction: BaseInstruction,
        time: f32,
    ) -> (Vec3, Quat) {
        match instruction {
            BaseInstruction::Back => (
                transform.rotation.mul_vec3(Vec3::X) * Self::VELOCITY * -1.0 * time,
                Quat::IDENTITY,
            ),
            BaseInstruction::Forward => (
                transform.rotation.mul_vec3(Vec3::X) * Self::VELOCITY * time,
                Quat::IDENTITY,
            ),
            BaseInstruction::Left => (
                Vec3::ZERO,
                Quat::from_euler(EulerRot::XYZ, 0.0, Self::VELOCITY * time, 0.0),
            ),
            BaseInstruction::Right => (
                Vec3::ZERO,
                Quat::from_euler(EulerRot::XYZ, 0.0, -Self::VELOCITY * time, 0.0),
            ),
        }
    }

    /// Updates Arenito's position given some time in seconds (`delta`).
    /// This method is suposed to be called every frame, where delta
    /// is the time between this frame's render and the previous.
    fn update_pos(&mut self, delta: f32, transform: &Transform) -> (Vec3, Quat) {
        let mut pos = Vec3::ZERO;
        let mut rot = Quat::IDENTITY;
        let mut delta = delta;

        if let Some((instr, rem_time)) = self.instruction_handler.current() {
            if delta > rem_time {
                // println!("Less than remaining time.");
                let (npos, nrot) = self.calculate_next_pos(transform, instr, rem_time);
                pos += npos;
                rot *= nrot;
                delta -= rem_time;
                self.instruction_handler.next();
            }

            match self.instruction_handler.current() {
                None => {}
                Some((instr, rem_time)) => {
                    let time = delta.min(rem_time);
                    // println!("executing for {}s", time);
                    let (npos, nrot) = self.calculate_next_pos(transform, instr, time);
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
        mut arenito_body: ParamSet<(
            Query<&mut Transform, With<ArenitoCompFrame>>,
            Query<&mut Transform, With<ArenitoCompBrush>>,
            Query<&mut Transform, With<ArenitoCompLeftWheel>>,
            Query<&mut Transform, With<ArenitoCompRightWheel>>,
        )>,
    ) {
        let mut arenito_frame = arenito_body.p0();
        let mut arenito_frame = arenito_frame.single_mut();
        arenito_frame.translation += pos_diff;
        arenito_frame.rotation *= rot_diff;

        let mut arenito_brush = arenito_body.p1();
        let mut arenito_brush = arenito_brush.single_mut();
        arenito_brush.rotate_local_z(-Self::BRUSH_SPEED * delta);

        // wheel rotation
        let mut l = 1.0;
        let mut r = 1.0;

        let t = if rot_diff == Quat::IDENTITY {
            pos_diff.length() * Self::VELOCITY
        } else {
            let (_, y, _) = rot_diff.to_euler(EulerRot::XYZ);
            l = if y > 0.0 { 1.0 } else { -1.0 };
            r = if y > 0.0 { -1.0 } else { 1.0 };

            rot_diff.mul_vec3(Vec3::X).length() * Self::VELOCITY
        };

        for mut wheel in arenito_body.p2().iter_mut() {
            wheel.rotate_local_z(-t * l);
        }
        for mut wheel in arenito_body.p3().iter_mut() {
            wheel.rotate_local_z(-t * r);
        }
    }
}

impl WithDistanceCollision for Arenito {
    fn get_pos(&self, transform: &Transform) -> Vec3 {
        transform.rotation.mul_vec3(self.brush_offset) + transform.translation
    }

    fn get_radius(&self) -> f32 {
        0.4
    }
}

/// Despawns cans when collided with Arenito
pub fn eat_cans(
    mut commands: Commands,
    arenito: Query<(&Arenito, &Transform)>,
    cans: Query<(&CanData, Entity, &Transform)>,
) {
    let (arenito, arenito_transform) = arenito.single();

    for (can, ent, can_transform) in cans.iter() {
        if arenito.collides_with_dist(can, arenito_transform, can_transform) {
            commands.entity(ent).despawn();
        }
    }
}

pub fn scan_distance(
    arenito: Query<(&Arenito, &Transform)>,
    obstacle: Query<(&Obstacle, &Handle<Mesh>, &Transform)>,
    mut proxs: Query<(&mut ProximitySensor, &Transform)>,
    meshes: Res<Assets<Mesh>>,
    mut gizmos: Gizmos,
) {
    let (_, arenito_transform) = arenito.single();

    for (mut prox, prox_transform) in proxs.iter_mut() {
        prox.reset();
        let prox_transform = prox_transform.from_parent(&arenito_transform);

        for (obstacle, obstacle_mesh, obstacle_transform) in obstacle.iter() {
            let obstacle_mesh = meshes.get(obstacle_mesh).unwrap();

            prox.collides_with_mesh(&prox_transform, obstacle, obstacle_mesh, obstacle_transform);
        }

        prox.draw_ray(&prox_transform, &mut gizmos);
    }
}
