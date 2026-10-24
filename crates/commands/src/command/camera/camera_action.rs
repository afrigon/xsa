use usage::Subcommands;

use super::{CameraLookAtCommand, CameraSnapCommand, CameraTargetCommand, CameraTeleportCommand};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum CameraAction {
    /// Orbit a body: a body id, next, previous or parent
    Target(CameraTargetCommand),
    /// Stand the debug camera on a body at a latitude and longitude
    Tp(CameraTeleportCommand),
    /// Turn the debug camera towards a body
    LookAt(CameraLookAtCommand),
    /// Save the rendered frame as a PNG, returning once the file is written
    Snap(CameraSnapCommand),
}

impl Routable for CameraAction {
    fn route(self) -> Route {
        match self {
            CameraAction::Target(command) => command.route(),
            CameraAction::Tp(command) => command.route(),
            CameraAction::LookAt(command) => command.route(),
            CameraAction::Snap(command) => command.route(),
        }
    }
}
