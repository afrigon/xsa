use std::io::{self, IsTerminal};

use tracing::Level;
use tracing_subscriber::EnvFilter;
use xsa_commands::console_log::ConsoleLog;

pub struct Logging {
    level: Level,
}

impl Logging {
    pub fn from_verbosity(verbose: u8, quiet: bool) -> Logging {
        let level = match verbose {
            _ if quiet => Level::ERROR,
            0 => Level::WARN,
            1 => Level::INFO,
            2 => Level::DEBUG,
            _ => Level::TRACE,
        };

        Logging { level }
    }

    // RUST_LOG, when set, overrides the level from -v and -q, e.g. RUST_LOG=xsa_client=debug.
    pub fn install(self) {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.level.as_str()));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(ConsoleLog::writer)
            .with_ansi(io::stderr().is_terminal())
            .init();
    }
}
