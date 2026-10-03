use xsa_commands::command::{CameraMode as CommandCameraMode, CameraModeCommand};

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::camera::CameraMode;

impl ClientCommandHandler for CameraModeCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let description = match self.mode {
            CommandCameraMode::Target => {
                app.set_camera_mode(CameraMode::Orbit);
                "camera: target"
            }
            CommandCameraMode::Debug => {
                app.set_camera_mode(CameraMode::Debug);
                "camera: debug"
            }
        };

        Ok(description.to_string())
    }
}
