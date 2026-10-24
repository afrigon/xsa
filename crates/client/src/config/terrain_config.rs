use xsa_config::{ConfigSection, ConfigVisitor};

use super::{TerrainLodConfig, TerrainStreamingConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainConfig {
    pub lod: TerrainLodConfig,
    pub streaming: TerrainStreamingConfig,
}

impl ConfigSection for TerrainConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.section("lod", &mut self.lod);
        visitor.section("streaming", &mut self.streaming);
    }
}
