use anyhow::Context;
use xsa_commands::command::CameraTargetCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;
use crate::camera::CameraMode;

impl ImmediateCommandHandler for CameraTargetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let world = app.world.as_ref().context("not joined to a simulation yet")?;
        let description = app.cameras.target(&self, world)?;
        app.set_camera_mode(CameraMode::Orbit);

        Ok(description)
    }
}
