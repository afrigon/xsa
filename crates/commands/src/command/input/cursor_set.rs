use usage::Args;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InputCursorSetCommand {
    #[usage(allow_negative_numbers, help = "Logical pixels from the window's left edge")]
    pub x: f64,
    #[usage(allow_negative_numbers, help = "Logical pixels from the window's top edge")]
    pub y: f64,
}

impl Routable for InputCursorSetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputCursorSet(self))
    }
}
