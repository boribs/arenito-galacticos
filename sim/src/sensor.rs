use crate::arenito::*;
use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use rand::{prelude::thread_rng, Rng};

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

/// This struct is going to be responsible for figuring out the relation
/// between something seen on the camera and it's "real" position.
/// TODO: This information will later be used to map the terrain and
/// decide the best path to the can deposit (and other things).
#[derive(Resource)]
pub struct ImageProcessor {
    // texture stuff
    pub image_handle: Option<Handle<Image>>,
    pub material_handle: Option<Handle<StandardMaterial>>,
    pub texture_width: u32,
    pub texture_height: u32,
    // camera data
    pub offset: Vec3,
    pub alpha: f32,
    pub va: f32,
    pub ha: f32,
    // projection stuff
    pub trapeze_long_side: f32,
    pub trapeze_short_side: f32,
    pub trapeze_height: f32,
}

impl ImageProcessor {
    /// Initializes image and material handlers.
    /// Must be called before doing any image processing.
    pub fn init(
        &mut self,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        images: &mut ResMut<Assets<Image>>,
    ) {
        let size = Extent3d {
            width: self.texture_width,
            height: self.texture_height,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);
        let image_handle = images.add(image);

        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(image_handle.clone()),
            reflectance: 0.02,
            unlit: true,
            ..default()
        });

        self.image_handle = Some(image_handle);
        self.material_handle = Some(material_handle);
    }

    /// Calculates long and short sides of the visible area, as well as height.
    /// Returns CameraArea.
    pub fn get_visible_area(&mut self) -> CameraArea {
        let area = CameraArea::from_img_processor(&self);
        let points = area.area_points();

        // unsure as tu why 0 - a, 1 - b, ... relation is broken
        // but after plotting `points`, this is how we calculate
        // long and short sides of the trapeze
        self.trapeze_long_side = points[0].distance(points[1]);
        self.trapeze_short_side = points[3].distance(points[2]);
        self.trapeze_height = {
            let long = points[0] - points[1];
            let short = points[3] - points[2];
            short.distance(long)
        };

        area
    }

impl Default for ImageProcessor {
    fn default() -> Self {
        ImageProcessor {
            image_handle: None,
            material_handle: None,
            texture_width: 0,
            texture_height: 0,
            offset: Vec3::ZERO,
            alpha: 0.0,
            ha: 0.0,
            va: 0.0,
            trapeze_long_side: 0.0,
            trapeze_short_side: 0.0,
            trapeze_height: 0.0,
        }
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
