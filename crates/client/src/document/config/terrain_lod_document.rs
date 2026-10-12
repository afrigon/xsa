use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues, NumberRange};
use crate::config::TerrainLodConfig;

// Vertices closer than a pixel apart add triangles without adding anything visible.
const MINIMUM_VERTEX_SPACING_PIXELS: f32 = 1.0;

pub struct TerrainLodDocument {
    vertex_spacing_pixels: Option<f32>,
    freeze: Option<bool>,
}

impl TerrainLodDocument {
    pub const VERTEX_SPACING: ConfigKey = ConfigKey::session(
        "render.lod.terrain.vertex-spacing",
        ConfigValueKind::Number {
            range: NumberRange::at_least(MINIMUM_VERTEX_SPACING_PIXELS),
        },
    );
    pub const FREEZE: ConfigKey = ConfigKey::session("render.lod.terrain.freeze", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> TerrainLodDocument {
        TerrainLodDocument {
            vertex_spacing_pixels: values.number(TerrainLodDocument::VERTEX_SPACING),
            freeze: values.bool(TerrainLodDocument::FREEZE),
        }
    }

    pub fn into_config(self, defaults: &TerrainLodConfig) -> TerrainLodConfig {
        TerrainLodConfig {
            vertex_spacing_pixels: self.vertex_spacing_pixels.unwrap_or(defaults.vertex_spacing_pixels),
            freeze: self.freeze.unwrap_or(defaults.freeze),
        }
    }

    pub fn write(config: &TerrainLodConfig, layer: &mut ConfigLayer) {
        layer.set(
            TerrainLodDocument::VERTEX_SPACING.path,
            ConfigValueKind::number(config.vertex_spacing_pixels),
        );
        layer.set(TerrainLodDocument::FREEZE.path, config.freeze.into());
    }
}
