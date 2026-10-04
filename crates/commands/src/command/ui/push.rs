use usage::Args;

use super::ViewControllerId;
use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfacePushCommand {
    #[usage(value_enum)]
    pub view_controller: ViewControllerId,
}

impl Routable for InterfacePushCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfacePush(self))
    }
}
