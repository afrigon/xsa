use usage::Args;

use super::{Routable, Route};

#[derive(Args)]
pub struct ExitCommand {}

impl Routable for ExitCommand {
    fn route(self) -> Route {
        Route::Exit
    }
}
