use crate::app::ClientWorld;
use crate::camera::Camera;
use crate::config::InputConfig;
use crate::input::Input;

// What controls read to move the camera for a frame.
pub struct ControlsContext<'a> {
    pub world: &'a ClientWorld,
    pub input: &'a Input,
    pub camera: &'a Camera,
    pub config: &'a InputConfig,
    pub delta_seconds: f64,
}
