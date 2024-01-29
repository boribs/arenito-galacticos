use crate::arenito::*;
use bevy::{prelude::*, render::view::screenshot::ScreenshotManager};
use memmap::MmapMut;
use rand::{prelude::thread_rng, Rng};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
};

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
        arenito.rot.mul_vec3(Vec3::X) + SensorError::default()
    }
}

/// Move instruction abstraction.
#[derive(Debug, Clone, PartialEq)]
pub enum SimInstruction {
    MoveForward,
    MoveLeft,
    MoveRight,
    MoveLongRight,
    ScreenShot,
}

/// Wrapper struct to store raw pointers to shared memory.
/// This is needed in order to be able to store pointers in `AISimMem`.
#[derive(Clone)]
pub struct AISimAddr(*mut u8);

// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for AISimAddr {}
unsafe impl Sync for AISimAddr {}

impl AISimAddr {
    pub fn set(&mut self, val: u8) {
        unsafe {
            *self.0 = val;
        }
    }

    pub fn write(&mut self, bytes: &Vec<u8>) {
        unsafe {
            for i in 0..bytes.len() {
                *(self.0.add(i)) = bytes[i];
            }
        }
    }

    pub fn get(&self) -> u8 {
        unsafe { *self.0 }
    }
}

/// Responsible for interacting with Arenito's AI process.
/// Communicates through shared file mapping.
///
/// The shared file block serves as both the communication
/// and the sync channel.
/// The first byte is used to syncrchronize the simulation,
/// as well as the AI, indicatin which process has write permission.
/// The rest of the block is to send data.
///
/// The sync flags (constants) indicate the steps of communication:
/// - AI_FRAME_REQUEST
/// - SIM_FRAME_WAIT
/// - AI_MOVE_INSTRUCTION
/// - SIM_AKNOWLEDGE_INSTRUCTION
///
/// ---
/// ## Memory footprint:
/// The first byte is always the synchronization byte.
/// The rest depend on the sync byte:
///
/// When sync is AI_MOVE_INSTRUCTION:
///   The next byte (second) is the movement instruction.
///
/// When sync is SIM_AKNOWLEDGE_INSTRUCTION, after AI_FRAME_REQUEST:
///   The following IMG_SIZE bytes are raw image data.
/// The image sent is of size (1024, 1024).
#[derive(Resource)]
pub struct AISimMem {
    sync_byte: AISimAddr,
    memspace: AISimAddr,
}

impl AISimMem {
    // sync constants
    const AI_FRAME_REQUEST: u8 = 1;
    const SIM_FRAME_WAIT: u8 = 2;
    const AI_MOVE_INSTRUCTION: u8 = 3;
    const SIM_AKNOWLEDGE_INSTRUCTION: u8 = 4;

    // movement instruction constants
    const MOV_FORWARD: u8 = b'a';
    const MOV_LEFT: u8 = b'i';
    const MOV_RIGHT: u8 = b'd';
    const MOV_LONG_RIGHT: u8 = b'D';

    // memory footprint
    // how much memory is used for synchronization
    const SYNC_SIZE: usize = 1;
    // min size required to store image, found experimentally
    #[cfg(target_os = "macos")]
    const IMG_SIZE: usize = 3_145_728;
    #[cfg(target_os = "windows")]
    const IMG_SIZE: usize = 786_432;
    // sync byte + img size
    pub const MIN_REQUIRED_MEMORY: usize = Self::SYNC_SIZE + Self::IMG_SIZE;
    pub const MMAP_FILENAME: &'static str = "file.mmap";

    pub fn new(mmap: &mut MmapMut) -> Self {
        unsafe {
            let ptr = mmap.as_mut_ptr();

            Self {
                sync_byte: AISimAddr(ptr),
                memspace: AISimAddr(ptr.add(1)),
            }
        }
    }

    pub fn create_shareable_file() -> File {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(Self::MMAP_FILENAME)
            .unwrap();
        file.seek(SeekFrom::Start(Self::MIN_REQUIRED_MEMORY as u64))
            .unwrap();
        file.write_all(&[0]).unwrap();
        file
    }

    /// Sets the sync flag of the mapping.
    fn set_sync_flag(&mut self, flag: u8) {
        self.sync_byte.set(flag);
    }

    /// Takes a screenshot of Arenito's Camera and writes it to the shared memory block.
    pub fn export_frame(
        &mut self,
        screenshot_manager: &mut ResMut<ScreenshotManager>,
        window: &Entity,
    ) {
        // prevent multiple screenshot requests
        self.set_sync_flag(AISimMem::SIM_FRAME_WAIT);

        // can't use directly `self.sync_byte`, thank you borrow checker.
        let mut sync_byte = self.sync_byte.clone();
        let mut memspace = self.memspace.clone();

        let _ =
            screenshot_manager.take_screenshot(*window, move |img| match img.try_into_dynamic() {
                Ok(dyn_img) => {
                    let img_raw = dyn_img.to_rgb8().into_raw();

                    // println!("raw len: {}", img_raw.len());
                    // println!("{}, {}", dyn_img.width(), dyn_img.height());

                    if img_raw.len() != AISimMem::IMG_SIZE {
                        panic!("different image size!");
                    }

                    memspace.write(&img_raw);
                    sync_byte.set(AISimMem::SIM_AKNOWLEDGE_INSTRUCTION);
                }
                Err(_) => {
                    println!("Cannot send screenshot!")
                }
            });
    }

    /// Returns the instruction for the simulation to execute.
    /// Returns None if there's none.
    ///
    /// If sync byte is `AI_MOVE_INSTRUCTION`:
    /// The next byte (memspace) is the movement instruction:
    /// - MOV_FORWARD
    /// - MOV_LEFT
    /// - MOV_RIGHT
    /// Any other memspace value will result in a None
    ///
    /// If sync byte is `AI_FRAME_REQUEST` no more bytes are checked.
    pub fn get_instruction(&self) -> Option<SimInstruction> {
        match self.sync_byte.get() {
            AISimMem::AI_FRAME_REQUEST => Some(SimInstruction::ScreenShot),
            AISimMem::AI_MOVE_INSTRUCTION => match self.memspace.get() {
                AISimMem::MOV_FORWARD => Some(SimInstruction::MoveForward),
                AISimMem::MOV_LEFT => Some(SimInstruction::MoveLeft),
                AISimMem::MOV_RIGHT => Some(SimInstruction::MoveRight),
                AISimMem::MOV_LONG_RIGHT => Some(SimInstruction::MoveLongRight),
                other => {
                    println!("Unrecognized movement instruction '{}'", other);
                    None
                }
            },
            _ => None,
        }
    }

    /// Sets the sync flag to `SIM_AKNOWLEDGE_INSTRUCTION`.
    /// Indicates to the AI that the simulation is done processing the message and
    /// is ready to read another instruction.
    ///
    /// Must be called after writing data.
    pub fn confirm_instruction(&mut self) {
        self.sync_byte.set(AISimMem::SIM_AKNOWLEDGE_INSTRUCTION);
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
        let mut arenito = Arenito::new();

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

#[cfg(test)]
mod ai_sim_mem_tests {
    use super::*;

    impl AISimMem {
        fn from_buf(buf: &mut Vec<u8>) -> Self {
            unsafe {
                Self {
                    sync_byte: AISimAddr(buf.as_mut_ptr()),
                    memspace: AISimAddr(buf.as_mut_ptr().add(1)),
                }
            }
        }
    }

    #[test]
    fn test_aisimaddr_set() {
        let mut buf: Vec<u8> = vec![0];
        let mut ptr = AISimAddr(buf.as_mut_ptr());
        ptr.set(45);

        assert_eq!(buf[0], 45);
    }

    #[test]
    fn test_aisimaddr_get() {
        let mut buf: Vec<u8> = vec![103];
        let ptr = AISimAddr(buf.as_mut_ptr());

        assert_eq!(ptr.get(), 103);
    }

    #[test]
    fn test_get_instruction_frame_request() {
        // mock buffer, to avoid actual shared memory
        let mut buf: Vec<u8> = vec![AISimMem::AI_FRAME_REQUEST, 0];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(Some(SimInstruction::ScreenShot), aisim.get_instruction());
    }

    #[test]
    fn test_get_instruction_frame_wait() {
        let mut buf: Vec<u8> = vec![AISimMem::SIM_FRAME_WAIT, 0];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(None, aisim.get_instruction());
    }

    #[test]
    fn test_get_instruction_move_instruction_forward() {
        let mut buf: Vec<u8> = vec![AISimMem::AI_MOVE_INSTRUCTION, AISimMem::MOV_FORWARD];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(Some(SimInstruction::MoveForward), aisim.get_instruction());
    }

    #[test]
    fn test_get_instruction_move_instruction_left() {
        let mut buf: Vec<u8> = vec![AISimMem::AI_MOVE_INSTRUCTION, AISimMem::MOV_LEFT];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(Some(SimInstruction::MoveLeft), aisim.get_instruction());
    }

    #[test]
    fn test_get_instruction_move_instruction_right() {
        let mut buf: Vec<u8> = vec![AISimMem::AI_MOVE_INSTRUCTION, AISimMem::MOV_RIGHT];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(Some(SimInstruction::MoveRight), aisim.get_instruction());
    }

    #[test]
    fn test_get_instruction_move_instruction_other_value_is_none() {
        let mut buf: Vec<u8> = vec![AISimMem::AI_MOVE_INSTRUCTION, 45];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(None, aisim.get_instruction());
    }

    #[test]
    fn test_get_instruction_aknowledge_instruction() {
        let mut buf: Vec<u8> = vec![AISimMem::SIM_AKNOWLEDGE_INSTRUCTION, 0];
        let aisim = AISimMem::from_buf(&mut buf);

        assert_eq!(None, aisim.get_instruction());
    }

    #[test]
    fn test_confirm_instruction() {
        let mut buf: Vec<u8> = vec![100, 101, 102, 103];
        let mut aisim = AISimMem::from_buf(&mut buf);

        aisim.confirm_instruction();
        assert_eq!(buf[0], AISimMem::SIM_AKNOWLEDGE_INSTRUCTION);
    }
}
