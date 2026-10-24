use anyhow::ensure;
use xsa_commands::command::CameraSpeedCommand;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

const DEBUG_CAMERA_SPEED_KEY: &str = "debug.camera.speed";

impl ImmediateCommandHandler for CameraSpeedCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let meters_per_second = self.speed.meters_per_second;
        ensure!(meters_per_second > 0.0, "the debug camera's speed must be above zero");

        app.config_store
            .set(DEBUG_CAMERA_SPEED_KEY, &meters_per_second.to_string(), false)?;
        app.reconfigure();

        Ok(format!("camera: debug speed {meters_per_second} m/s"))
    }
}
