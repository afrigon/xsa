use usage::Subcommands;

use super::{
    InterfaceHideCommand, InterfacePopCommand, InterfacePushCommand, InterfaceSetCommand, InterfaceShowCommand,
    InterfaceToggleCommand,
};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum InterfaceAction {
    /// Push a view controller over the current one
    Push(InterfacePushCommand),
    /// Go back to the view controller below the current one
    Pop(InterfacePopCommand),
    /// Replace every view controller with this one
    Set(InterfaceSetCommand),
    /// Show an overlay
    Show(InterfaceShowCommand),
    /// Hide an overlay
    Hide(InterfaceHideCommand),
    /// Show or hide an overlay
    Toggle(InterfaceToggleCommand),
}

impl Routable for InterfaceAction {
    fn route(self) -> Route {
        match self {
            InterfaceAction::Push(command) => command.route(),
            InterfaceAction::Pop(command) => command.route(),
            InterfaceAction::Set(command) => command.route(),
            InterfaceAction::Show(command) => command.route(),
            InterfaceAction::Hide(command) => command.route(),
            InterfaceAction::Toggle(command) => command.route(),
        }
    }
}
