use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::InputConfig;

pub struct InputDocument {
    enabled: Option<bool>,
}

impl InputDocument {
    pub const ENABLED: ConfigKey = ConfigKey::session("input.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> InputDocument {
        InputDocument {
            enabled: values.bool(InputDocument::ENABLED),
        }
    }

    pub fn into_config(self, defaults: &InputConfig) -> InputConfig {
        InputConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
        }
    }

    pub fn write(config: &InputConfig, layer: &mut ConfigLayer) {
        layer.set(InputDocument::ENABLED.path, config.enabled.into());
    }
}
