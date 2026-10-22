mod camera_action;
mod camera_mode;
mod look_at;
mod mode;
mod snap;
mod speed;
mod target;
mod teleport;

pub use camera_action::CameraAction;
pub use camera_mode::CameraMode;
pub use look_at::CameraLookAtCommand;
pub use mode::CameraModeCommand;
pub use snap::CameraSnapCommand;
pub use speed::CameraSpeedCommand;
pub use target::CameraTargetCommand;
pub use teleport::CameraTeleportCommand;

use usage::Args;

use super::{Routable, Route};

#[derive(Args)]
pub struct CameraCommand {
    #[usage(subcommand)]
    pub action: CameraAction,
}

impl Routable for CameraCommand {
    fn route(self) -> Route {
        self.action.route()
    }
}
