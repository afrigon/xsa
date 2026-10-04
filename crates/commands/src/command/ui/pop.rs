use usage::Args;

use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfacePopCommand {}

impl Routable for InterfacePopCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfacePop(self))
    }
}
