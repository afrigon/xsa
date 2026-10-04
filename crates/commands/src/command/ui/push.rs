use usage::Args;

use super::Screen;
use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfacePushCommand {
    #[usage(value_enum)]
    pub screen: Screen,
}

impl Routable for InterfacePushCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfacePush(self))
    }
}
