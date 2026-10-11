use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::LensConfig;

pub struct LensDocument {
    enabled: Option<bool>,
}

impl LensDocument {
    pub const ENABLED: ConfigKey = ConfigKey::saved("render.lens.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> LensDocument {
        LensDocument {
            enabled: values.bool(LensDocument::ENABLED),
        }
    }

    pub fn into_config(self, defaults: &LensConfig) -> LensConfig {
        LensConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
        }
    }

    pub fn write(config: &LensConfig, layer: &mut ConfigLayer) {
        layer.set(LensDocument::ENABLED.path, config.enabled.into());
    }
}
