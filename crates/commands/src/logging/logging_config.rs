use xsa_config::{ConfigSection, ConfigVisitor};
use xsa_logging::LogFilter;

use super::ShellLoggingConfig;
use crate::shell::Transcript;

const NAME: &str = "logging";
const FILTER: &str = "filter";

// The section names itself so its filter key can be set from the command line at startup.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoggingConfig {
    pub filter: LogFilter,
    pub shell: ShellLoggingConfig,
}

impl LoggingConfig {
    pub fn filter_key() -> String {
        format!("{NAME}.{FILTER}")
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
