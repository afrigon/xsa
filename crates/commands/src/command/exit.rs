use usage::Args;

use super::{Availability, Routable, Route};

#[derive(Args)]
pub struct ExitCommand {}

impl Routable for ExitCommand {
    fn route(self) -> Route {
        Route::Exit
    }

    fn availability(&self) -> Availability {
        Availability::Anywhere
    }
}
