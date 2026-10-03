use usage::Subcommands;

use super::{CameraLookAtCommand, CameraModeCommand, CameraTargetCommand};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum CameraAction {
    /// Switch between the target camera and the debug fly camera
    Mode(CameraModeCommand),
    /// Orbit a body: a body id, next, previous or parent
    Target(CameraTargetCommand),
    /// Turn the debug camera towards a body
    LookAt(CameraLookAtCommand),
}

impl Routable for CameraAction {
    fn route(self) -> Route {
        match self {
            CameraAction::Mode(command) => command.route(),
            CameraAction::Target(command) => command.route(),
            CameraAction::LookAt(command) => command.route(),
        }
    }
}
