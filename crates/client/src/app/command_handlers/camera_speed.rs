use anyhow::Context;
use xsa_commands::command::CameraSpeedCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for CameraSpeedCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let cameras = app.cameras.as_mut().context("the cameras are not ready")?;

        cameras.set_debug_speed(self.speed)
    }
}
