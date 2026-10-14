mod attitude;
mod camera_mode;
pub mod debug;
pub mod orbit;

pub use attitude::Attitude;
pub use camera_mode::CameraMode;

use glam::camera::rh::proj::vulkan;
use glam::{DQuat, DVec3, Mat4, Vec4};

// Reverse-Z float depth keeps the same relative precision at every distance, so one near plane serves all scales.
const NEAR_PLANE: f32 = 0.1;

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
    pub aspect_ratio: f32,
}

impl Camera {
    pub fn new(horizontal_fov: f32, aspect_ratio: f32) -> Self {
        Self {
            position: DVec3::ZERO,
            orientation: DQuat::IDENTITY,
            horizontal_fov,
            aspect_ratio,
        }
    }

    pub fn view_rotation(&self) -> Mat4 {
        VIEW_FROM_CAMERA * Mat4::from_quat(self.orientation.inverse().as_quat())
    }

    pub fn vertical_fov(&self) -> f32 {
        2.0 * ((self.horizontal_fov / 2.0).tan() / self.aspect_ratio).atan()
    }

    pub fn narrowest_half_fov(&self) -> f32 {
        self.horizontal_fov.min(self.vertical_fov()) / 2.0
    }

    pub fn clip_from_view(&self) -> Mat4 {
        vulkan::perspective_infinite_reverse(self.vertical_fov(), self.aspect_ratio, NEAR_PLANE)
    }
}
