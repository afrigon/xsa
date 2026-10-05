use usage::Subcommands;

use super::{CameraLookAtCommand, CameraModeCommand, CameraSnapCommand, CameraTargetCommand};
use crate::command::{Availability, Routable, Route};

#[derive(Subcommands)]
pub enum CameraAction {
    /// Switch between the target camera and the debug fly camera
    Mode(CameraModeCommand),
    /// Orbit a body: a body id, next, previous or parent
    Target(CameraTargetCommand),
    /// Turn the debug camera towards a body
    LookAt(CameraLookAtCommand),
    /// Save the rendered frame as a PNG, returning once the file is written
    Snap(CameraSnapCommand),
}

impl Routable for CameraAction {
    fn route(self) -> Route {
        match self {
            CameraAction::Mode(command) => command.route(),
            CameraAction::Target(command) => command.route(),
            CameraAction::LookAt(command) => command.route(),
            CameraAction::Snap(command) => command.route(),
        }
    }

    fn availability(&self) -> Availability {
        match self {
            CameraAction::Mode(command) => command.availability(),
            CameraAction::Target(command) => command.availability(),
            CameraAction::LookAt(command) => command.availability(),
            CameraAction::Snap(command) => command.availability(),
        }
    }
}
