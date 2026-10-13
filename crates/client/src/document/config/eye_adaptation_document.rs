use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::EyeAdaptationConfig;

pub struct EyeAdaptationDocument {
    enabled: Option<bool>,
}

impl EyeAdaptationDocument {
    pub const ENABLED: ConfigKey = ConfigKey::session("render.eye-adaptation.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> EyeAdaptationDocument {
        EyeAdaptationDocument {
            enabled: values.bool(EyeAdaptationDocument::ENABLED),
        }
    }

    pub fn into_config(self, defaults: &EyeAdaptationConfig) -> EyeAdaptationConfig {
        EyeAdaptationConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
        }
    }

    pub fn write(config: &EyeAdaptationConfig, layer: &mut ConfigLayer) {
        layer.set(EyeAdaptationDocument::ENABLED.path, config.enabled.into());
    }
}
