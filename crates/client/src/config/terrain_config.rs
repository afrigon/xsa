use xsa_config::{ConfigSection, ConfigVisitor};

use super::{TerrainHeightConfig, TerrainLodConfig, TerrainStreamingConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainConfig {
    pub lod: TerrainLodConfig,
    pub streaming: TerrainStreamingConfig,
    pub height: TerrainHeightConfig,
}

impl ConfigSection for TerrainConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.section("lod", &mut self.lod);
        visitor.section("streaming", &mut self.streaming);
        visitor.section("height", &mut self.height);
    }
}
