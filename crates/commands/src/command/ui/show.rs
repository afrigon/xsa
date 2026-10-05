use usage::Args;

use super::Overlay;
use crate::command::{Availability, ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfaceShowCommand {
    #[usage(value_enum)]
    pub overlay: Overlay,
}

impl Routable for InterfaceShowCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfaceShow(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
