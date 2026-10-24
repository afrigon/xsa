use usage::Subcommands;

use super::{InputCursorMoveCommand, InputCursorSetCommand};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum InputCursorAction {
    /// Put the cursor at a position in the window
    Set(InputCursorSetCommand),
    /// Move the mouse by an offset: the cursor, or the view while the mouse is captured
    Move(InputCursorMoveCommand),
}

impl Routable for InputCursorAction {
    fn route(self) -> Route {
        match self {
            InputCursorAction::Set(command) => command.route(),
            InputCursorAction::Move(command) => command.route(),
        }
    }
}
