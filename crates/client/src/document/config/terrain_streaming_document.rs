use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::TerrainStreamingConfig;

pub struct TerrainStreamingDocument {
    freeze: Option<bool>,
}

impl TerrainStreamingDocument {
    pub const FREEZE: ConfigKey = ConfigKey::session("resources.streaming.terrain.freeze", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> TerrainStreamingDocument {
        TerrainStreamingDocument {
            freeze: values.bool(TerrainStreamingDocument::FREEZE),
        }
    }

    pub fn into_config(self, defaults: &TerrainStreamingConfig) -> TerrainStreamingConfig {
        TerrainStreamingConfig {
            freeze: self.freeze.unwrap_or(defaults.freeze),
        }
    }

    pub fn write(config: &TerrainStreamingConfig, layer: &mut ConfigLayer) {
        layer.set(TerrainStreamingDocument::FREEZE.path, config.freeze.into());
    }
}
