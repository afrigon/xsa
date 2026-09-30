pub mod debug;

use glam::camera::rh::proj::vulkan;
use glam::{DQuat, DVec3, Mat4, Vec4};

const VIEW_FROM_CAMERA: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, -1.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::W,
);

pub struct Camera {
    pub position: DVec3,
    pub orientation: DQuat,
    pub horizontal_fov: f32,
    pub near: f32,
}

impl Camera {
    pub fn view_rotation(&self) -> Mat4 {
        VIEW_FROM_CAMERA * Mat4::from_quat(self.orientation.inverse().as_quat())
    }

    pub fn clip_from_view(&self, aspect_ratio: f32) -> Mat4 {
        let vertical_fov = 2.0 * ((self.horizontal_fov / 2.0).tan() / aspect_ratio).atan();
        vulkan::perspective_infinite_reverse(vertical_fov, aspect_ratio, self.near)
    }
}
