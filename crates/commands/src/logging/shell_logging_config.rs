use xsa_config::{ConfigSection, ConfigVisitor, Inheritable};
use xsa_logging::LogFilter;

use crate::shell::Transcript;

#[derive(Clone, Debug, PartialEq)]
pub struct ShellLoggingConfig {
    pub enabled: bool,
    pub filter: Inheritable<LogFilter>,
    pub ipc: Transcript,
    pub script: Transcript,
}

impl Default for ShellLoggingConfig {
    fn default() -> Self {
        ShellLoggingConfig {
            enabled: true,
            filter: Inheritable::Inherit,
            ipc: Transcript::default(),
            script: Transcript::default(),
        }
    }
}

impl ConfigSection for ShellLoggingConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.value("filter", &mut self.filter);
        visitor.section("ipc", &mut self.ipc);
        visitor.section("script", &mut self.script);
    }
}
