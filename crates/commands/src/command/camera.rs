mod camera_action;
mod look_at;
mod snap;
mod target;
mod teleport;

pub use camera_action::CameraAction;
pub use look_at::CameraLookAtCommand;
pub use snap::CameraSnapCommand;
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
