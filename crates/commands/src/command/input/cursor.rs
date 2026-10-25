use usage::Args;

use super::InputCursorAction;
use crate::command::{Routable, Route};

#[derive(Args)]
pub struct InputCursorCommand {
    #[usage(subcommand)]
    pub action: InputCursorAction,
}

impl Routable for InputCursorCommand {
    fn route(self) -> Route {
        self.action.route()
    }
}
