use super::{NumberRange, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::BloomConfig;

pub struct BloomDocument {
    enabled: Option<bool>,
    strength: Option<f32>,
}

impl BloomDocument {
    pub const ENABLED: ConfigKey = ConfigKey::saved("render.bloom.enabled", ConfigValueKind::Bool);
    pub const STRENGTH: ConfigKey = ConfigKey::saved(
        "render.bloom.strength",
        ConfigValueKind::Number {
            range: NumberRange::at_least(0.0),
        },
    );

    pub fn read(values: &ConfigValues) -> BloomDocument {
        BloomDocument {
            enabled: values.bool(BloomDocument::ENABLED),
            strength: values.number(BloomDocument::STRENGTH),
        }
    }

    pub fn into_config(self, defaults: &BloomConfig) -> BloomConfig {
        BloomConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
            strength: self.strength.unwrap_or(defaults.strength),
        }
    }

    pub fn write(config: &BloomConfig, layer: &mut ConfigLayer) {
        layer.set(BloomDocument::ENABLED.path, config.enabled.into());
        layer.set(BloomDocument::STRENGTH.path, ConfigValueKind::number(config.strength));
    }
}
