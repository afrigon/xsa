use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, PartialEq)]
pub struct DebugConfig {
    pub wireframe: bool,
}

impl ConfigSection for DebugConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("wireframe", &mut self.wireframe);
    }
}
