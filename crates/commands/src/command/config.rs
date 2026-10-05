mod config_action;
mod get;
mod reload;
mod save;
mod set;
mod toggle;

pub use config_action::ConfigAction;
pub use get::ConfigGetCommand;
pub use reload::ConfigReloadCommand;
pub use save::ConfigSaveCommand;
pub use set::ConfigSetCommand;
pub use toggle::ConfigToggleCommand;

use usage::Args;

use super::{Availability, Routable, Route};

#[derive(Args)]
pub struct ConfigCommand {
    #[usage(subcommand)]
    pub action: ConfigAction,
}

impl Routable for ConfigCommand {
    fn route(self) -> Route {
        self.action.route()
    }

    fn availability(&self) -> Availability {
        self.action.availability()
    }
}
