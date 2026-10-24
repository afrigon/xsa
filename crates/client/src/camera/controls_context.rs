use crate::app::ClientWorld;
use crate::camera::Camera;
use crate::config::DebugCameraConfig;
use crate::input::Input;

// What controls read to move the camera for a frame.
pub struct ControlsContext<'a> {
    pub world: &'a ClientWorld,
    pub input: &'a Input,
    pub camera: &'a Camera,
    pub debug_camera: &'a DebugCameraConfig,
    pub delta_seconds: f64,
}
