use crate::arenito::*;
use bevy::{prelude::*, render::view::screenshot::ScreenshotManager};
use rand::{prelude::thread_rng, Rng};
use std::{fs::File, io::prelude::*, thread, thread::JoinHandle};

const INSTRUCTION_PIPE_PATH: &str = "../pipes/instrpipe";
const IMAGE_PIPE_PATH: &str = "../pipes/imgpipe";

/// This trait aims to unify the calculation of a direction vector from
/// the output of MPU6050's gyroscope.
/// Tailored specifically for this simulator's application it's assumed
/// that the X+ axis points (initially) forwards and it controls the roll
/// of Arenito.
///
/// To determine the direction only the yaw and the pitch are considered.
/// That means that the y (yaw) and z (pitch) components of the gyro values
/// are used.
pub trait FromGyro {
    fn from_gyro(gyro: &Vec3) -> Vec3;
}

impl FromGyro for Vec3 {
    /// Creates a unit vector from a rotation vector.
    fn from_gyro(gyro: &Vec3) -> Vec3 {
        Vec3::new(gyro.y.cos(), gyro.z.sin(), gyro.y.sin())
    }
}

/// This struct's purpose is to generalize "Error Vector" generation.
/// Since this is a needed step on simulating sensors.
struct SensorError;
impl SensorError {
    const DEFAULT_MIN: f32 = -0.05;
    const DEFAULT_MAX: f32 = 0.05;

    /// Returns a Vec3 with random values in the range min..max.
    pub fn vec(min: f32, max: f32) -> Vec3 {
        let mut rng = thread_rng();
        Vec3::new(
            rng.gen_range(min..max),
            rng.gen_range(min..max),
            rng.gen_range(min..max),
        )
    }

    /// Returns the default (kinda like std::default::Default) vector
    /// with an error range DEFAULT_MIN..DEFAULT_MAX.
    pub fn default() -> Vec3 {
        SensorError::vec(SensorError::DEFAULT_MIN, SensorError::DEFAULT_MAX)
    }
}

/// This struct is responsible for the simulations of all sensors related to Arenito.
/// This sensor simulation is based on the MPU6050 Chip, which includes an acceleromter
/// and a gyroscope.
/// The outputs of this simulator are trying to be as similar as posible to this:
/// (https://randomnerdtutorials.com/arduino-mpu-6050-accelerometer-gyroscope/)
pub struct MPU6050;

impl MPU6050 {
    // This is the upper bound for the accelerometer readings.
    // Depends entirely on the hardware.
    // Acceleration reads CAN NOT be higher than this.
    pub const ACCELERATION_MAX: f32 = 9.8; // 1g!

    /// Gets Arenito's "real" acceleration and converts it
    /// to something the real accelerometer would return:
    /// A value between 0 and 1024 that represents the magnitude
    /// of the acceleration on each axis.
    pub fn read_acc(arenito: &Arenito) -> Vec3 {
        // get acceleration value
        // convert to absolute value
        let mut acc = arenito.acc.abs();

        // add error
        acc += SensorError::default();

        // interpolate each value between [0, 1024],
        // considering that Sensor::ACCELERATION_MAX maps to 1024.
        (acc.abs() * 1024.0 / MPU6050::ACCELERATION_MAX).ceil()
    }

    /// Gets Arenito's "real" rotation.
    /// Technically the sensor outputs rotational speed, but I'm
    /// too lazy to simulate that.
    /// This implementation skips all the math needed to convert
    /// from rotational speed to "current rotation" altogether.
    pub fn read_rot(arenito: &Arenito) -> Vec3 {
        arenito.rot + SensorError::default()
    }
}

/// Move instruction abstraction.
#[derive(Debug, Clone)]
pub enum SimInstruction {
    Move(ArenitoState),
    ScreenShot,
}

/// Responsible for interacting with Arenito's AI process.
/// Reads pipe and determines move instruction.
#[derive(Resource)]
pub struct SimInterface {
    thread: Option<JoinHandle<String>>,
}

impl SimInterface {
    pub fn new() -> Self {
        Self { thread: None }
    }

    /// Takes a screenshot of Arenito's Camera and pipes it.
    fn export_image(
        screenshot_manager: &mut  ResMut<ScreenshotManager>,
        window: &Entity,
    ) {
        let _ = screenshot_manager.take_screenshot(*window, move |img| match img.try_into_dynamic() {
            Ok(dyn_img) => {
                let img = dyn_img.to_rgb8();

                let pipe = File::create(IMAGE_PIPE_PATH);
                let _ = pipe
                    .as_ref()
                    .expect("Could not open pipe")
                    .write_all(&img.into_raw());
            }
            Err(_) => {println!("Cannot save screenshot!")},
        });
    }

    /// Reads input from pipe and parses. Returns movement direction.
    pub fn listen(
        &mut self,
        screenshot_manager: &mut ResMut<ScreenshotManager>,
        window: &Entity,
    ) -> Option<ArenitoState> {
        let input = self.read_pipe();
        if input.is_some() {
            let input = input.unwrap();
            let instr = SimInterface::parse_input(&input);
            if instr.is_err() {
                println!("Cannot parse: '{}'", &input);
                return None;
            }

            match instr.unwrap() {
                SimInstruction::Move(dir) => {
                    return Some(dir);
                }
                SimInstruction::ScreenShot => {
                    SimInterface::export_image(screenshot_manager, window);
                    return None;
                }
            };
        }

        None
    }

    /// Reads pipe content.
    fn read_pipe(&mut self) -> Option<String> {
        if self.thread.is_none() {
            self.thread = Some(thread::spawn(|| {
                let mut cin = String::new();
                let pipe = File::open(INSTRUCTION_PIPE_PATH);

                let _ = pipe
                    .as_ref()
                    .expect("can't open pipe!")
                    .read_to_string(&mut cin);

                cin
            }));
        } else if self.thread.as_ref().unwrap().is_finished() {
            // https://stackoverflow.com/questions/57670145/how-to-store-joinhandle-of-a-thread-to-close-it-later
            let input = self.thread.take().map(JoinHandle::join).unwrap().unwrap();
            self.thread = None;
            return Some(input);
        }

        None
    }

    /// Parses input string.
    /// Expected input has the following syntax:
    /// mv:<dir>
    /// where <dir> can be one of the following:
    ///     fw - forward
    ///     l  - left
    ///     r  - right
    ///
    /// Will also accept
    /// ss
    /// requesting Arenito's Camera's screen shot
    fn parse_input(input: &String) -> Result<SimInstruction, ()> {
        if input.starts_with("mv:") {
            let (_, dir) = input.split_at(3);
            return match dir {
                "fw" => Ok(SimInstruction::Move(ArenitoState::FORWARD)),
                "l" => Ok(SimInstruction::Move(ArenitoState::LEFT)),
                "r" => Ok(SimInstruction::Move(ArenitoState::RIGHT)),
                _ => Err(())
            }
        } else if input == "ss" {
            return Ok(SimInstruction::ScreenShot);
        }

        Err(())
    }
}

#[cfg(test)]
mod sensor_read_tests {
    use super::*;

    fn acc_within_value(vec: &Vec3) {
        // println!("{}", vec);
        assert!(vec.x >= 0.0 && vec.x <= 1024.0);
        assert!(vec.y >= 0.0 && vec.y <= 1024.0);
        assert!(vec.z >= 0.0 && vec.z <= 1024.0);
    }

    #[test]
    fn sensor_acc_reads_dont_go_to_negative_values() {
        let mut rng = thread_rng();
        let mut arenito = Arenito::test();

        for _ in 0..100 {
            arenito.acc = Vec3::new(
                rng.gen_range(-2.1..2.1),
                rng.gen_range(-2.1..2.1),
                rng.gen_range(-2.1..2.1),
            );
            let read = MPU6050::read_acc(&arenito);
            acc_within_value(&read);
        }
    }

    // No idea how to or what to test for gyro reads...
}
