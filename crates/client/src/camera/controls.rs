use super::{Anchor, CameraPose, ControlsContext};
use crate::config::DebugCameraConfig;
use crate::input::Input;

// How input moves the camera. Each kind of controls keeps its own state; switching kinds replaces the controls.
pub trait Controls {
    fn update(&mut self, context: &ControlsContext) -> CameraPose;

    fn anchor(&self) -> Anchor;

    fn wants_mouse_capture(&self, input: &Input) -> Option<bool>;

    fn name(&self) -> &'static str;

    // Meters per second, for controls that fly at a set speed.
    fn speed(&self, debug_camera: &DebugCameraConfig) -> Option<f64>;
}
