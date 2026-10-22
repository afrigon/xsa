use std::io;

use anstream::{AutoStream, ColorChoice};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;

use crate::colored_writer::ColoredWriter;
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
        for<'writer> <Writer as MakeWriter<'writer>>::Writer: Send + 'static,
    {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.verbosity.level().as_str()));

        let colored = AutoStream::choice(&io::stderr()) != ColorChoice::Never;

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(ColoredWriter::new(writer, colored))
            .with_ansi(true)
            .init();
    }
}
