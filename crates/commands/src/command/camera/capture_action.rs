use usage::Subcommands;

use super::{CameraCaptureStartCommand, CameraCaptureStopCommand};

#[derive(Subcommands)]
pub enum CameraCaptureAction {
    /// Start capturing a video until `camera capture stop`
    Start(CameraCaptureStartCommand),
    /// Stop the camera's video capture, returning once the file is written
    Stop(CameraCaptureStopCommand),
}
