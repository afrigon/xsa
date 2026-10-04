use usage::Args;

use super::Screen;
use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfaceSetCommand {
    #[usage(value_enum)]
    pub screen: Screen,
}

impl Routable for InterfaceSetCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfaceSet(self))
    }
}
