use std::io::{self, IsTerminal};

use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, reload};

use crate::{LogFilter, LogFilterHandle};

pub struct Logging {
    filter: LogFilter,
}

impl Logging {
    pub fn new(filter: LogFilter) -> Logging {
        Logging { filter }
    }

    pub fn install<Writer>(self, writer: Writer) -> LogFilterHandle
    where
        Writer: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
    {
        let (filter, handle) = reload::Layer::new(self.filter.env_filter());

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(writer).with_ansi(io::stderr().is_terminal()))
            .init();

        LogFilterHandle::new(handle)
    }
}
