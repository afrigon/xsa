use usage::Args;

use super::ViewControllerId;
use crate::command::{Availability, ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfacePushCommand {
    #[usage(value_enum)]
    pub view_controller: ViewControllerId,
}

impl Routable for InterfacePushCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfacePush(self))
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
