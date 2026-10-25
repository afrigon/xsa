use usage::Subcommands;

use super::{
    InputCursorCommand, InputDownCommand, InputPressCommand, InputResetCommand, InputScrollCommand, InputUpCommand,
};
use crate::command::{Routable, Route};

#[derive(Subcommands)]
pub enum InputAction {
    /// Press a chord's buttons and hold them
    Down(InputDownCommand),
    /// Release a chord's buttons
    Up(InputUpCommand),
    /// Press a chord's buttons, then release them
    Press(InputPressCommand),
    /// Release every held button
    Reset(InputResetCommand),
    /// Move the cursor
    Cursor(InputCursorCommand),
    /// Scroll the mouse wheel or touchpad
    Scroll(InputScrollCommand),
}

impl Routable for InputAction {
    fn route(self) -> Route {
        match self {
            InputAction::Down(command) => command.route(),
            InputAction::Up(command) => command.route(),
            InputAction::Press(command) => command.route(),
            InputAction::Reset(command) => command.route(),
            InputAction::Cursor(command) => command.route(),
            InputAction::Scroll(command) => command.route(),
        }
    }
}
