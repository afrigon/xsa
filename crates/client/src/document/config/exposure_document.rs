use super::{ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues, NumberRange};
use crate::config::{ExposureConfig, ExposureMode};

// Below the light of the night sky, above the brightest sunlit scene.
const MINIMUM_EV100: f32 = -15.0;
const MAXIMUM_EV100: f32 = 25.0;

pub struct ExposureDocument {
    mode: Option<ExposureMode>,
    ev100: Option<f32>,
}

impl ExposureDocument {
    pub const MODE: ConfigKey = ConfigKey::session(
        "render.exposure.mode",
        ConfigValueKind::Choice {
            names: <ExposureMode as ConfigChoice>::names,
        },
    );
    pub const EV100: ConfigKey = ConfigKey::session(
        "render.exposure.ev100",
        ConfigValueKind::Number {
            range: NumberRange::between(MINIMUM_EV100, MAXIMUM_EV100),
        },
    );

    pub fn read(values: &ConfigValues) -> ExposureDocument {
        ExposureDocument {
            mode: values.choice(ExposureDocument::MODE),
            ev100: values.number(ExposureDocument::EV100),
        }
    }

    pub fn into_config(self, defaults: &ExposureConfig) -> ExposureConfig {
        ExposureConfig {
            mode: self.mode.unwrap_or(defaults.mode),
            ev100: self.ev100.unwrap_or(defaults.ev100),
        }
    }

    pub fn write(config: &ExposureConfig, layer: &mut ConfigLayer) {
        layer.set(ExposureDocument::MODE.path, config.mode.config_name().into());
        layer.set(ExposureDocument::EV100.path, ConfigValueKind::number(config.ev100));
    }
}
