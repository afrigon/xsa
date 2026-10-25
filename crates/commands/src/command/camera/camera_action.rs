use usage::Subcommands;

use super::{CameraCaptureCommand, CameraLookAtCommand, CameraTargetCommand, CameraTeleportCommand};

#[derive(Subcommands)]
pub enum CameraAction {
    /// Orbit a body by id, or the next, previous or parent one with --next, --previous or --parent
    Target(CameraTargetCommand),
    /// Stand the debug camera on a body at a latitude and longitude
    Tp(CameraTeleportCommand),
    /// Turn the debug camera towards a body
    LookAt(CameraLookAtCommand),
    /// Save the rendered frame as a PNG, or a video with --duration or start, returning once the file is written
    Capture(CameraCaptureCommand),
}
