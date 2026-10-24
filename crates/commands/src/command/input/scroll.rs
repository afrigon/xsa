use usage::Args;
use xsa_input::ScrollDelta;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InputScrollCommand {
    #[usage(
        allow_negative_numbers,
        help = "How far, positive away from the user: wheel notches (3) or touchpad pixels (120px)"
    )]
    pub amount: ScrollDelta,
}

impl Routable for InputScrollCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputScroll(self))
    }
}
