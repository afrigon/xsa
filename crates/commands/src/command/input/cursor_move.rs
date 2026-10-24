use usage::Args;

use crate::command::{ClientCommand, Routable, Route};
use crate::value::{Duration, EasingName};

#[derive(Args)]
pub struct InputCursorMoveCommand {
    #[usage(allow_negative_numbers, help = "Logical pixels to the right")]
    pub dx: f64,
    #[usage(allow_negative_numbers, help = "Logical pixels down")]
    pub dy: f64,
    #[usage(
        long,
        help = "Spread the motion over this long, e.g. 500ms or 2s; at once without it"
    )]
    pub duration: Option<Duration>,
    #[usage(long, value_enum, default_value_t = EasingName::Linear, default = "linear", help = "How the motion is spread over its duration")]
    pub easing: EasingName,
}

impl Routable for InputCursorMoveCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputCursorMove(self))
    }
}
