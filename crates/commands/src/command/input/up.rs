use usage::Args;
use xsa_input::InputChord;

use crate::command::{ClientCommand, Routable, Route};
use crate::completion::complete_input_chord;

#[derive(Args)]
pub struct InputUpCommand {
    #[usage(complete = complete_input_chord, help = "The buttons to release, e.g. w or shift+tab")]
    pub chord: InputChord,
}

impl Routable for InputUpCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InputUp(self))
    }
}
