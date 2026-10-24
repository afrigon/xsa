use usage::Args;

use crate::command::{ClientCommand, Routable, Route};
use crate::value::{Duration, EasingName};

#[derive(Args)]
pub struct InputCursorSetCommand {
    #[usage(allow_negative_numbers, help = "Logical pixels from the window's left edge")]
    pub x: f64,
    #[usage(allow_negative_numbers, help = "Logical pixels from the window's top edge")]
    pub y: f64,
    #[usage(
        long,
        help = "Spread the motion over this long, e.g. 500ms or 2s; at once without it"
    )]
    pub duration: Option<Duration>,
    #[usage(long, value_enum, default_value_t = EasingName::Linear, default = "linear", help = "How the motion is spread over its duration")]
    pub easing: EasingName,
}

impl Routable for InputCursorSetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputCursorSet(self))
    }
}
