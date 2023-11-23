use crate::{arenito::*, static_shape::CameraArea};
use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, PrimitiveTopology, TextureDescriptor, TextureDimension, TextureFormat,
        TextureUsages,
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
    pub cam_area: CameraArea,
    // distance from arenito's center to the trapeze's short side's
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

    /// Calculates dimensions of visible area.
    /// Returns self.
    pub fn compute_area(&mut self, arenito: &Arenito) -> &Self {
        self.cam_area.compute_area(arenito.cam_offset);
        self
    }

    /// Returns the mesh of the trapeze (Arenito's camera's visible area).
    pub fn get_mesh(&self) -> Mesh {
        let mut points = self.cam_area.points.clone();
        points.push(points[0].clone());

        let normals = vec![[1.0, 1.0, 1.0]; 5];
        let uvs = vec![[1.0, 1.0]; 5];

        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }

    /// Calculates a point's position on the visible area, based on it's position on
    /// the texture.
    /// The returned position is on a theoretical trapeze, which is aligned horizontally
    /// on the origin, it's short side is on the y axis and it's long side is
    /// somewhere on the y+ plane.
    /// ```txt
    ///
    ///             |
    ///     +-------|-------+        ↑
    ///      \   p  |      /         | y > 0
    ///       \     |     /          |
    ///        +----O----+    --------------> x = 0
    ///             |
    ///             |
    /// ```
    pub fn point_to_trapeze(&self, x: i32, z: i32) -> Vec2 {
        let x = x - (self.texture_width as i32 / 2);

        // horizontal position relative to center
        let k = 2.0 * x as f32 / self.texture_width as f32;
        // vertical position relative to bottom
        let l = z as f32 / self.texture_height as f32;

        // calculate position on trapeze
        let (a, b) = (
            Vec2::new(
                self.cam_area.long_side * k / 2.0,
                self.cam_area.height as f32,
            ),
            Vec2::new(self.cam_area.short_side * k / 2.0, 0.0),
        );

        // calculate slope like this to facilitate checking for infinite slope
        let m = a - b;
        // y = wherever the point is vertically,
        // that is: trapeze height * vertical position relative to bottom (l):
        // y = trapeze_heigh * l
        let y = self.cam_area.height * l;

        if m.x == 0.0 {
            // check for infinite slope
            // this means that the point is in the middle
            Vec2::new(0.0, y)
        } else {
            // no infinite slope
            // considering that the line ecuation goes y - y_0 = m(x - x_0)
            // and we're starting from `b`, we know that y_0 and x_0 are
            // the components of `b`.
            //
            // given that we know `y`, `y_0` = 0 and `x_0` = b.x we can plug them
            // on the equation:
            // y - 0 = mx - mb.x
            // y = mx - mb.x
            // mx = y + mb.x
            // x = y/m + b.x

            let m = m.y / m.x;
            Vec2::new((y / m) + b.x, y)
        }
    }

    /// Proyects the point on the theoretical trapeze into the "real" one.
    /// That is, moves the theoretical trapeze into the real one's position.
    pub fn project_point(&self, point: Vec2, arenito: &Arenito) -> Vec3 {
        // starting from Arenito's position, we know that it's visible area's
        // center (the middle of the short side of the trapeze) is located
        // `dist_to_trapeze` forward, so we just translate the point (d, 0, 0)
        // where d = `dist_to_trapeze` and rotate by arenito's rotation.

        let point = Vec3::new(point.y, 0.0, point.x) + self.cam_area.center;

        let c = Vec3::new(arenito.center.x, 0.0, arenito.center.z);
        let q = Quat::from_euler(EulerRot::XYZ, 0.0, arenito.rot.y, 0.0);

        q.mul_vec3(point) + c
    }
}

impl Default for ImageProcessor {
    fn default() -> Self {
        ImageProcessor {
            image_handle: None,
            material_handle: None,
            texture_width: 0,
            texture_height: 0,
            cam_area: CameraArea::default(),
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

#[cfg(test)]
mod image_processor_tests {
    use super::*;

    /// Helper function to initialize ImageProcessor quickly.
    fn get_im(a: &Arenito, ha: f32, va: f32, alpha: f32, tw: u32, th: u32) -> ImageProcessor {
        let mut im = ImageProcessor {
            cam_area: CameraArea::new(ha, va, alpha),
            texture_width: tw,
            texture_height: th,
            ..default()
        };
        im.compute_area(a);
        im
    }

    #[test]
    fn point_projected_in_trapeze_1() {
        let im = get_im(&Arenito::new(), 45.0, 45.0, -40.0, 512, 512);
        // starting with a 512 x 512px image, projecting a point
        // on it's center should result in a point in the middle
        // of the trapeze.
        // if the trapeze has a long side of 3.3088057, a short side
        // of 1.121719 and a height of 3.4463365, then the point
        // should be on (0, 1.0935433 [height / 2]).

        assert_eq!(
            im.point_to_trapeze(256, 256),
            Vec2::new(0.0, im.cam_area.height / 2.0)
        )
    }

    #[test]
    fn point_projected_in_trapeze_2() {
        let im = get_im(&Arenito::new(), 45.0, 45.0, -40.0, 512, 512);

        assert_eq!(
            im.point_to_trapeze(153, 256),
            Vec2::new(-0.4456485, im.cam_area.height / 2.0)
        )
    }

    #[test]
    fn point_projected_in_trapeze_3() {
        let im = get_im(&Arenito::new(), 45.0, 45.0, -40.0, 512, 512);

        assert_eq!(
            im.point_to_trapeze(345, 210),
            Vec2::new(0.35091836, 1.4135364)
        )
    }

    #[test]
    fn point_from_theoretical_to_real_1() {
        let a = Arenito::new();
        let im = get_im(&a, 45.0, 45.0, -40.0, 512, 512);
        let p = im.point_to_trapeze(256, 256);

        assert_eq!(
            im.project_point(p, &a),
            im.cam_area.center + Vec3::new(im.cam_area.height / 2.0, 0.0, 0.0)
        )
    }

    #[test]
    fn point_from_theoretical_to_real_2() {
        let a = Arenito::new();
        let im = get_im(&a, 45.0, 45.0, -40.0, 512, 512);
        let p = im.point_to_trapeze(345, 210);

        assert_eq!(
            im.project_point(p, &a),
            im.cam_area.center + Vec3::new(im.cam_area.height * 0.41015625, 0.0, 0.35091836)
        )
    }

    #[test]
    fn point_from_theoretical_to_real_3() {
        let mut a = Arenito::new();
        let im = get_im(&a, 45.0, 45.0, -40.0, 512, 512);
        let p = im.point_to_trapeze(345, 210);

        // move Arenito
        let fwd = Vec3::new(10.0, 0.0, 0.0);
        a.center += fwd;

        assert_eq!(
            im.project_point(p, &a),
            im.cam_area.center + Vec3::new(im.cam_area.height * 0.41015625, 0.0, 0.35091836) + fwd
        )
    }

    #[test]
    fn point_from_theoretical_to_real_4() {
        let mut a = Arenito::new();
        let im = get_im(&a, 45.0, 45.0, -40.0, 512, 512);
        let p = im.point_to_trapeze(256, 256);

        // start from non-rotated point
        let expected = im.cam_area.center + Vec3::new(im.cam_area.height / 2.0, 0.0, 0.0);
        // then rotate
        let expected =
            Quat::from_euler(EulerRot::XYZ, 0.0, f32::to_radians(15.0), 0.0).mul_vec3(expected);

        a.rot = Vec3::new(0.0, f32::to_radians(15.0), 0.0);
        assert_eq!(im.project_point(p, &a), expected)
    }

    #[test]
    fn point_from_theoretical_to_real_5() {
        let mut a = Arenito::new();
        let im = get_im(&a, 45.0, 45.0, -40.0, 512, 512);
        let p = im.point_to_trapeze(256, 256);

        // move Arenito
        let fwd = Vec3::new(-5.0, 0.0, 0.0);
        a.center += fwd;

        // start from non-rotated point
        let expected = im.cam_area.center + Vec3::new(im.cam_area.height / 2.0, 0.0, 0.0);
        // then rotate
        let expected = Quat::from_euler(EulerRot::XYZ, 0.0, f32::to_radians(-45.0), 0.0)
            .mul_vec3(expected)
            + fwd;

        a.rot = Vec3::new(0.0, f32::to_radians(-45.0), 0.0);
        assert_eq!(im.project_point(p, &a), expected)
    }
}
