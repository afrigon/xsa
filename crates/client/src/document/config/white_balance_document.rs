use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::WhiteBalanceConfig;

pub struct WhiteBalanceDocument {
    enabled: Option<bool>,
}

impl WhiteBalanceDocument {
    pub const ENABLED: ConfigKey = ConfigKey::session("render.white-balance.enabled", ConfigValueKind::Bool);

    pub fn read(values: &ConfigValues) -> WhiteBalanceDocument {
        WhiteBalanceDocument {
            enabled: values.bool(WhiteBalanceDocument::ENABLED),
        }
    }

    pub fn into_config(self, defaults: &WhiteBalanceConfig) -> WhiteBalanceConfig {
        WhiteBalanceConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
        }
    }

    pub fn write(config: &WhiteBalanceConfig, layer: &mut ConfigLayer) {
        layer.set(WhiteBalanceDocument::ENABLED.path, config.enabled.into());
    }
}
