use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainScatterConfig {
    // Off, scatters neither place nor draw their props.
    pub enabled: bool,
}

impl ConfigSection for TerrainScatterConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
    }
}
