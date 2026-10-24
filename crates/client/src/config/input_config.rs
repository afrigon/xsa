use xsa_config::{ConfigSection, ConfigVisitor};

use super::BindConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct InputConfig {
    // Off, the camera ignores the mouse and keyboard and never captures the mouse; key binds still run.
    pub enabled: bool,
    pub bind: BindConfig,
}

impl ConfigSection for InputConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.section("bind", &mut self.bind);
    }
}
