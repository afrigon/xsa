use super::{CameraLookAtCommand, CameraModeCommand, CameraTargetCommand};

pub enum ClientCommand {
    CameraMode(CameraModeCommand),
    CameraTarget(CameraTargetCommand),
    CameraLookAt(CameraLookAtCommand),
}
