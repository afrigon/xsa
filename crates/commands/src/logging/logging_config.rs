use xsa_config::{ConfigSection, ConfigStore, ConfigValue, ConfigVisitor};
use xsa_logging::LogFilter;

use super::ShellLoggingConfig;
use crate::shell::Transcript;

const NAME: &str = "logging";
const FILTER: &str = "filter";

// The section names itself so its filter key is known when the command line overrides it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoggingConfig {
    pub filter: LogFilter,
    pub shell: ShellLoggingConfig,
}

impl LoggingConfig {
    // For the session only, so a flag or RUST_LOG never reaches the file.
    pub fn override_filter<T: ConfigSection + Default + Clone>(
        store: &mut ConfigStore<T>,
        filter: &LogFilter,
    ) -> anyhow::Result<()> {
        store.set(&format!("{NAME}.{FILTER}"), &filter.text(), false)
    }

    pub fn shell_filter(&self) -> LogFilter {
        if self.shell.enabled {
            self.shell.filter.or(&self.filter)
        } else {
            LogFilter::off()
        }
    }

    pub fn ipc_transcript(&self) -> Transcript {
        self.shell.ipc.within(self.shell.enabled)
    }

    pub fn script_transcript(&self) -> Transcript {
        self.shell.script.within(self.shell.enabled)
    }
}

impl ConfigSection for LoggingConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.group(NAME, |visitor| {
            visitor.value(FILTER, &mut self.filter);
            visitor.section("shell", &mut self.shell);
        });
    }
}
