use usage::Args;

use super::{Routable, Route};
use crate::value::Duration;

#[derive(Args)]
pub struct WaitCommand {
    #[usage(help = "How long to wait in real time, e.g. 50ms, 2s or 60t (ticks)")]
    pub duration: Duration,
}

impl Routable for WaitCommand {
    fn route(self) -> Route {
        Route::Wait(self)
    }
}
