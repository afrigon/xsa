use std::time::Duration;

use xsa_config::{ConfigSection, ConfigVisitor};

use super::{TerrainHeightConfig, TerrainLodConfig, TerrainScatterConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainConfig {
    pub lod: TerrainLodConfig,
    pub height: TerrainHeightConfig,
    pub scatter: TerrainScatterConfig,
    // How long a terrain map keeps the finer levels the camera no longer needs, so returning soon costs nothing.
    pub texture_retention: Duration,
}

impl ConfigSection for TerrainConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.section("lod", &mut self.lod);
        visitor.section("height", &mut self.height);
        visitor.section("scatter", &mut self.scatter);
        visitor.value("texture-retention", &mut self.texture_retention);
    }
}
