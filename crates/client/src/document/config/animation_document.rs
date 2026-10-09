use super::{ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues, NumberRange};
use crate::config::AnimationConfig;

pub struct AnimationDocument {
    duration_scale: Option<f32>,
}

impl AnimationDocument {
    pub const DURATION_SCALE: ConfigKey = ConfigKey::session(
        "animations.duration-scale",
        ConfigValueKind::Number {
            range: NumberRange::at_least(0.0),
        },
    );

    pub fn read(values: &ConfigValues) -> AnimationDocument {
        AnimationDocument {
            duration_scale: values.number(AnimationDocument::DURATION_SCALE),
        }
    }

    pub fn into_config(self, defaults: &AnimationConfig) -> AnimationConfig {
        AnimationConfig {
            duration_scale: self.duration_scale.unwrap_or(defaults.duration_scale),
        }
    }

    pub fn write(config: &AnimationConfig, layer: &mut ConfigLayer) {
        layer.set(
            AnimationDocument::DURATION_SCALE.path,
            ConfigValueKind::number(config.duration_scale),
        );
    }
}
