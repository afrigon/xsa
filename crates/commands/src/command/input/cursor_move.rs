use usage::Args;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InputCursorMoveCommand {
    #[usage(allow_negative_numbers, help = "Logical pixels to the right")]
    pub dx: f64,
    #[usage(allow_negative_numbers, help = "Logical pixels down")]
    pub dy: f64,
}

impl Routable for InputCursorMoveCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputCursorMove(self))
    }
}
