mod hide;
mod interface_action;
mod overlay;
mod pop;
mod push;
mod set;
mod show;
mod toggle;
mod view_controller_id;

pub use hide::InterfaceHideCommand;
pub use interface_action::InterfaceAction;
pub use overlay::Overlay;
pub use pop::InterfacePopCommand;
pub use push::InterfacePushCommand;
pub use set::InterfaceSetCommand;
pub use show::InterfaceShowCommand;
pub use toggle::InterfaceToggleCommand;
pub use view_controller_id::ViewControllerId;

use usage::Args;

use super::{Routable, Route};

#[derive(Args)]
pub struct InterfaceCommand {
    #[usage(subcommand)]
    pub action: InterfaceAction,
}

impl Routable for InterfaceCommand {
    fn route(self) -> Route {
        self.action.route()
    }
}
