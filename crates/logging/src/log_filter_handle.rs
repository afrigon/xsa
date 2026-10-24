use tracing_subscriber::{EnvFilter, Registry, reload};

use crate::LogFilter;

// Swaps the filter of the installed subscriber while the process runs.
#[derive(Clone)]
pub struct LogFilterHandle {
    handle: reload::Handle<EnvFilter, Registry>,
}

impl LogFilterHandle {
    pub(crate) fn new(handle: reload::Handle<EnvFilter, Registry>) -> LogFilterHandle {
        LogFilterHandle { handle }
    }

    pub fn apply(&self, filter: &LogFilter) {
        if let Err(err) = self.handle.reload(filter.env_filter()) {
            tracing::warn!("the log filter could not change: {err}");
        }
    }
}
