use xsa_config::{ConfigSection, ConfigVisitor, NumberRange};

// Vertices closer than a pixel apart add triangles without adding anything visible.
const MINIMUM_VERTEX_SPACING_PIXELS: f32 = 1.0;

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainLodConfig {
    pub vertex_spacing_pixels: f32,
    // Tiles stop splitting and merging, so the current selection can be inspected from elsewhere.
    pub freeze: bool,
}

impl ConfigSection for TerrainLodConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.number(
            "vertex-spacing",
            &mut self.vertex_spacing_pixels,
            NumberRange::at_least(MINIMUM_VERTEX_SPACING_PIXELS),
        );
        visitor.value("freeze", &mut self.freeze);
    }
}
