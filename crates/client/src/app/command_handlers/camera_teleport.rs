use anyhow::Context;
use xsa_commands::command::CameraTeleportCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;
use crate::camera::CameraMode;

impl ImmediateCommandHandler for CameraTeleportCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let world = app.world.as_ref().context("not joined to a simulation yet")?;
        let cameras = app.cameras.as_ref().context("the cameras are not ready")?;
        // Read before the switch: in orbit mode the camera is around the body it orbits, not the nearest one.
        let around = cameras.reference_body(world);
        app.set_camera_mode(CameraMode::Debug);
        let world = app.world.as_ref().context("not joined to a simulation yet")?;
        let cameras = app.cameras.as_mut().context("the cameras are not ready")?;

        cameras.teleport(&self, around, world)
    }
}
