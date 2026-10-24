use xsa_commands::logging::LoggingConfig;
use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DedicatedConfig {
    pub logging: LoggingConfig,
}

impl ConfigSection for DedicatedConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        self.logging.visit(visitor);
    }
}
