use std::io::{self, IsTerminal};

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;

use crate::verbosity::Verbosity;

pub struct Logging {
    verbosity: Verbosity,
}

impl Logging {
    pub fn new(verbosity: Verbosity) -> Logging {
        Logging { verbosity }
    }

    // RUST_LOG, when set, overrides the level from -v and -q, e.g. RUST_LOG=xsa_client=debug.
    pub fn install<Writer>(self, writer: Writer)
    where
        Writer: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
    {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.verbosity.level().as_str()));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(writer)
            .with_ansi(io::stderr().is_terminal())
            .init();
    }
}
