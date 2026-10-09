use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues, NumberRange};
use crate::config::AdaptationConfig;

const MAXIMUM_HALF_LIFE_SECONDS: f32 = 10.0;

pub struct AdaptationDocument {
    dark_to_light: Option<f32>,
    light_to_dark: Option<f32>,
}

impl AdaptationDocument {
    pub const DARK_TO_LIGHT: ConfigKey = ConfigKey::session(
        "render.exposure.adaptation.dark-to-light",
        ConfigValueKind::Number {
            range: NumberRange::between(0.0, MAXIMUM_HALF_LIFE_SECONDS),
        },
    );
    pub const LIGHT_TO_DARK: ConfigKey = ConfigKey::session(
        "render.exposure.adaptation.light-to-dark",
        ConfigValueKind::Number {
            range: NumberRange::between(0.0, MAXIMUM_HALF_LIFE_SECONDS),
        },
    );

    pub fn read(values: &ConfigValues) -> AdaptationDocument {
        AdaptationDocument {
            dark_to_light: values.number(AdaptationDocument::DARK_TO_LIGHT),
            light_to_dark: values.number(AdaptationDocument::LIGHT_TO_DARK),
        }
    }

    pub fn into_config(self, defaults: &AdaptationConfig) -> AdaptationConfig {
        AdaptationConfig {
            dark_to_light_half_life_seconds: self.dark_to_light.unwrap_or(defaults.dark_to_light_half_life_seconds),
            light_to_dark_half_life_seconds: self.light_to_dark.unwrap_or(defaults.light_to_dark_half_life_seconds),
        }
    }

    pub fn write(config: &AdaptationConfig, layer: &mut ConfigLayer) {
        layer.set(
            AdaptationDocument::DARK_TO_LIGHT.path,
            ConfigValueKind::number(config.dark_to_light_half_life_seconds),
        );
        layer.set(
            AdaptationDocument::LIGHT_TO_DARK.path,
            ConfigValueKind::number(config.light_to_dark_half_life_seconds),
        );
    }
}
