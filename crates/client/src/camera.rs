mod anchor;
mod any_controls;
mod attitude;
mod camera_pose;
mod camera_switch;
mod controls;
mod controls_context;
mod debug;
mod field_of_view;
mod orbit;
mod transition;

pub use anchor::Anchor;
pub use any_controls::AnyControls;
pub use attitude::Attitude;
pub use camera_pose::CameraPose;
pub use camera_switch::CameraSwitch;
pub use controls::Controls;
pub use controls_context::ControlsContext;
pub use debug::DebugControls;
pub use field_of_view::FieldOfView;
pub use orbit::OrbitControls;
pub use transition::CameraTransition;

use glam::camera::rh::proj::vulkan;
use glam::{DQuat, DVec3, Mat4, Vec3, Vec4};

// Reverse-Z float depth keeps the same relative precision at every distance, so one near plane serves all scales.
const NEAR_PLANE: f32 = 0.1;
const CLEAR_COLOR: Vec3 = Vec3::ZERO;

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
    // Linear; shows wherever nothing is drawn.
    pub clear_color: Vec3,
    // Whether the controls keep the camera out of the ground.
    pub collision: bool,
}

impl Camera {
    pub fn new(horizontal_fov: f32, aspect_ratio: f32, collision: bool) -> Self {
        Self {
            position: DVec3::ZERO,
            orientation: DQuat::IDENTITY,
            horizontal_fov,
            aspect_ratio,
            clear_color: CLEAR_COLOR,
            collision,
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
