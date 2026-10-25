mod config_action;
mod get;
mod load;
mod save;
mod set;
mod settings;
mod toggle;
mod unset;

pub use config_action::ConfigAction;
pub use get::ConfigGetCommand;
pub use load::ConfigLoadCommand;
pub use save::ConfigSaveCommand;
pub use set::ConfigSetCommand;
use settings::Settings;
pub use toggle::ConfigToggleCommand;
pub use unset::ConfigUnsetCommand;

use usage::Args;

use super::{Routable, Route};

#[derive(Args)]
pub struct ConfigCommand {
    #[usage(subcommand)]
    pub action: ConfigAction,
}

impl Routable for ConfigCommand {
    fn route(self) -> Route {
        self.action.route()
    }
}
