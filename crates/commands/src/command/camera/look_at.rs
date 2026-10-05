use usage::Args;

use crate::command::{Availability, ClientCommand, Routable, Route};
use crate::completion::complete_target;
use crate::value::Target;

#[derive(Args)]
pub struct CameraLookAtCommand {
    #[usage(complete = complete_target)]
    pub target: Target,
}

impl Routable for CameraLookAtCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::CameraLookAt(self))
    }

    fn availability(&self) -> Availability {
        Availability::InGame
    }
}
