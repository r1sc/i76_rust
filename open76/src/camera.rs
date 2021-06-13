use std::f64::consts::PI;

use glam::{Quat, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::new(3337.0, 10.71, 35872.0),
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    pub fn turn(&mut self, x_delta: f32, y_delta: f32, secs: f32) {
        self.pitch -= y_delta * secs;
        self.yaw -= x_delta * secs;
    }

    pub fn translate(&mut self, z_delta: f32, x_delta: f32) {
        let deg_to_rad = (PI / 180.0) as f32;

        let quat = Quat::from_rotation_y((self.yaw + 180.0) * deg_to_rad)
            * Quat::from_rotation_x((self.pitch + 180.0) * deg_to_rad);

        let disp = (quat * -Vec3::Z * z_delta) + (quat * Vec3::X * x_delta);

        self.position.x -= disp.x;
        self.position.z -= disp.z;
        self.position.y += disp.y;
    }
}
