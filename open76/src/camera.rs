use std::f32::consts::PI;

use glam::{Mat4, Quat, Vec3, vec3};

pub struct Camera {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

const DEG_TO_RAD: f32 = PI / 180.0;

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::default(),
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    pub fn get_view(&self) -> Mat4 {
        let yaw_quat = Quat::from_axis_angle(Vec3::Y, self.yaw * DEG_TO_RAD);
        let pitch_quat = Quat::from_axis_angle(Vec3::X, self.pitch * DEG_TO_RAD);
        let rotation = pitch_quat * yaw_quat;
        Mat4::from_quat(rotation) * Mat4::from_translation(-self.position)
    }

    pub fn turn(&mut self, x_delta: f32, y_delta: f32, secs: f32) {
        self.yaw -= x_delta * secs;
        self.pitch -= y_delta * secs;
    }

    pub fn translate(&mut self, x_delta: f32, z_delta: f32) {
        let yaw_quat = Quat::from_axis_angle(Vec3::Y, self.yaw * DEG_TO_RAD);
        let pitch_quat = Quat::from_axis_angle(Vec3::X, self.pitch * DEG_TO_RAD);
        let rotation = (pitch_quat * yaw_quat).inverse();
        
        let forward = rotation * Vec3::NEG_Z;
        let side = rotation * Vec3::X;

        self.position += forward * z_delta + side * x_delta;
    }
}
