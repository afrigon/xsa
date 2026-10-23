use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainStreamingConfig {
    // Terrain pages stay as they are: nothing loads or is evicted while tiles keep requesting.
    pub freeze: bool,
}

impl ConfigSection for TerrainStreamingConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("freeze", &mut self.freeze);
    }
}
