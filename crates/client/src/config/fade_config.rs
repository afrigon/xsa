use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, PartialEq)]
pub struct FadeConfig {
    // Off, meshes pop in and out where their fades reach half instead of dissolving.
    pub enabled: bool,
}

impl ConfigSection for FadeConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
    }
}
