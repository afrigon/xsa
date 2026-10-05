use usage::Args;

use super::Overlay;
use crate::command::{Availability, ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfaceToggleCommand {
    #[usage(value_enum)]
    pub overlay: Overlay,
}

impl Routable for InterfaceToggleCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfaceToggle(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
