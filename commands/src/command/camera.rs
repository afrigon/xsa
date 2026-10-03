mod camera_action;
mod camera_mode;
mod look_at;
mod mode;
mod target;

pub use camera_action::CameraAction;
pub use camera_mode::CameraMode;
pub use look_at::CameraLookAtCommand;
pub use mode::CameraModeCommand;
pub use target::CameraTargetCommand;

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
