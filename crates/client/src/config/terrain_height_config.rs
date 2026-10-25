use xsa_config::{ConfigSection, ConfigVisitor, NumberRange};

// How the terrain's measured heights are drawn: off draws a smooth sphere, and the scale exaggerates relief and any
// mismatch between tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainHeightConfig {
    pub enabled: bool,
    pub scale: f32,
}

impl TerrainHeightConfig {
    // What the drawn heights are multiplied by.
    pub fn drawn_scale(&self) -> f32 {
        if self.enabled { self.scale } else { 0.0 }
    }
}

impl ConfigSection for TerrainHeightConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.number("scale", &mut self.scale, NumberRange::at_least(0.0));
    }
}
