use usage::Args;

use super::Overlay;
use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfaceHideCommand {
    #[usage(value_enum)]
    pub overlay: Overlay,
}

impl Routable for InterfaceHideCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfaceHide(self))
    }
}
