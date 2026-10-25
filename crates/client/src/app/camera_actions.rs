use anyhow::anyhow;
use xsa_commands::command::CameraAction;
use xsa_commands::value::CameraId;

use super::App;
use super::client_command_handler::ClientCommandHandler;
use super::command_progress::CommandProgress;

impl App {
    pub(super) fn start_camera(&mut self, camera: CameraId, action: CameraAction) -> CommandProgress {
        if !camera.is_main() {
            return CommandProgress::Done(Err(anyhow!("there is no camera {camera}: main is the only camera")));
        }

        match action {
            CameraAction::Target(command) => command.start(self),
            CameraAction::Tp(command) => command.start(self),
            CameraAction::LookAt(command) => command.start(self),
            CameraAction::Capture(command) => command.start(self),
        }
    }
}
