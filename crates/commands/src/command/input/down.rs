use usage::Args;
use xsa_input::InputChord;

use crate::command::{ClientCommand, Routable, Route};
use crate::completion::complete_input_chord;

#[derive(Args)]
pub struct InputDownCommand {
    #[usage(complete = complete_input_chord, help = "The buttons to press and hold, e.g. w, shift+tab or ctrl+mouse-left")]
    pub chord: InputChord,
}

impl Routable for InputDownCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputDown(self))
    }
}
