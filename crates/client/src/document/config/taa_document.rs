use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::TaaConfig;

pub struct TaaDocument {
    enabled: Option<bool>,
}

impl TaaDocument {
    pub const ENABLED: ConfigKey = ConfigKey::saved("render.taa.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> TaaDocument {
        TaaDocument {
            enabled: values.bool(TaaDocument::ENABLED),
        }
    }

    pub fn into_config(self, defaults: &TaaConfig) -> TaaConfig {
        TaaConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
        }
    }

    pub fn write(config: &TaaConfig, layer: &mut ConfigLayer) {
        layer.set(TaaDocument::ENABLED.path, config.enabled.into());
    }
}
