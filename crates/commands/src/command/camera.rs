mod camera_action;
mod capture;
mod capture_action;
mod capture_start;
mod capture_stop;
mod look_at;
mod target;
mod target_selection;
mod teleport;

pub use camera_action::CameraAction;
pub use capture::{CameraCaptureCommand, DEFAULT_CAPTURE_FPS};
pub use capture_action::CameraCaptureAction;
pub use capture_start::CameraCaptureStartCommand;
pub use capture_stop::CameraCaptureStopCommand;
pub use look_at::CameraLookAtCommand;
pub use target::CameraTargetCommand;
pub use target_selection::TargetSelection;
pub use teleport::CameraTeleportCommand;

use usage::Args;

use super::{ClientCommand, Routable, Route};
use crate::value::CameraId;

#[derive(Args)]
pub struct CameraCommand {
    #[usage(long, default_value_t = CameraId::main(), default = "main", help = "The camera to act on")]
    pub id: CameraId,
    #[usage(subcommand)]
    pub action: CameraAction,
}

impl Routable for CameraCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::Camera {
            camera: self.id,
            action: self.action,
        })
    }
}
