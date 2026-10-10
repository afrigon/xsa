use xsa_packs::Id;

use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::LensConfig;

pub struct LensDocument {
    enabled: Option<bool>,
    profile: Option<Id>,
}

impl LensDocument {
    pub const ENABLED: ConfigKey = ConfigKey::saved("render.lens.enabled", ConfigValueKind::Bool);
    pub const PROFILE: ConfigKey = ConfigKey::session("render.lens.profile", ConfigValueKind::Id);

    pub fn read(values: &ConfigValues) -> LensDocument {
        LensDocument {
            enabled: values.bool(LensDocument::ENABLED),
            profile: values.id(LensDocument::PROFILE),
        }
    }

    pub fn into_config(self, defaults: &LensConfig) -> LensConfig {
        LensConfig {
            enabled: self.enabled.unwrap_or(defaults.enabled),
            profile: self.profile.unwrap_or_else(|| defaults.profile.clone()),
        }
    }

    pub fn write(config: &LensConfig, layer: &mut ConfigLayer) {
        layer.set(LensDocument::ENABLED.path, config.enabled.into());
        layer.set(LensDocument::PROFILE.path, config.profile.to_string().into());
    }
}
