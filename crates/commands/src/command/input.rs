mod cursor;
mod cursor_action;
mod cursor_move;
mod cursor_set;
mod down;
mod input_action;
mod press;
mod reset;
mod scroll;
mod up;

pub use cursor::InputCursorCommand;
pub use cursor_action::InputCursorAction;
pub use cursor_move::InputCursorMoveCommand;
pub use cursor_set::InputCursorSetCommand;
pub use down::InputDownCommand;
pub use input_action::InputAction;
pub use press::InputPressCommand;
pub use reset::InputResetCommand;
pub use scroll::InputScrollCommand;
pub use up::InputUpCommand;

use usage::Args;

use super::{Routable, Route};

#[derive(Args)]
pub struct InputCommand {
    #[usage(subcommand)]
    pub action: InputAction,
}

impl Routable for InputCommand {
    fn route(self) -> Route {
        self.action.route()
    }
}
