use usage::Args;

use super::InterfaceElement;
use crate::command::{ClientCommand, Routable, Route};

#[derive(Args)]
pub struct InterfaceToggleCommand {
    #[usage(
        help = "An overlay to show or hide, or a view controller to push over the others or pop when it is on top: debug-overlay, pause-menu, …"
    )]
    pub element: InterfaceElement,
}

impl Routable for InterfaceToggleCommand {
    fn route(self) -> Route {
        Route::Client(ClientCommand::InterfaceToggle(self))
    }
}
