use usage::Args;
use xsa_input::{InputChord, ScrollDelta};

use crate::command::{ClientCommand, Routable, Route};
use crate::completion::complete_input_chord;
use crate::value::{Duration, EasingName};

#[derive(Args)]
pub struct InputScrollCommand {
    #[usage(
        allow_negative_numbers,
        help = "How far, positive away from the user: wheel notches (3) or touchpad pixels (120px)"
    )]
    pub amount: ScrollDelta,
    #[usage(
        long,
        help = "Spread the motion over this long, e.g. 500ms or 2s; at once without it"
    )]
    pub duration: Option<Duration>,
    #[usage(long, value_enum, default_value_t = EasingName::Linear, default = "linear", help = "How the motion is spread over its duration")]
    pub easing: EasingName,
    #[usage(long, complete = complete_input_chord, help = "Buttons held during the motion, e.g. mouse-left to drag")]
    pub hold: Option<InputChord>,
}

impl Routable for InputScrollCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputScroll(self))
    }
}
