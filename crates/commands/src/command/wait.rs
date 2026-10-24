use anyhow::Context;
use usage::Args;

use super::{Routable, Route};
use crate::value::Duration;

#[derive(Args)]
pub struct WaitCommand {
    #[usage(help = "How long to wait in real time, e.g. 50ms, 2s or 60t (ticks)")]
    pub duration: Duration,
}

impl WaitCommand {
    pub fn wait(&self) -> anyhow::Result<std::time::Duration> {
        let seconds = self.duration.duration().seconds;

        std::time::Duration::try_from_secs_f64(seconds)
            .ok()
            .context("a wait cannot be negative")
    }
}

impl Routable for WaitCommand {
    fn route(self) -> Route {
        Route::Wait(self)
    }
}
