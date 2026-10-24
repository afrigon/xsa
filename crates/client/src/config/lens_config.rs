use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, PartialEq)]
pub struct LensConfig {
    pub enabled: bool,
}

impl ConfigSection for LensConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
    }
}
