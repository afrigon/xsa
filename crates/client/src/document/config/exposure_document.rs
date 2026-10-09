use super::{AdaptationDocument, ConfigChoice, ConfigKey, ConfigLayer, ConfigValueKind, ConfigValues};
use crate::config::{ExposureConfig, ExposureMode};

pub struct ExposureDocument {
    mode: Option<ExposureMode>,
    ev100: Option<f32>,
    compensation: Option<f32>,
    adaptation: AdaptationDocument,
}

impl ExposureDocument {
    pub const MODE: ConfigKey = ConfigKey::session(
        "render.exposure.mode",
        ConfigValueKind::Choice {
            names: <ExposureMode as ConfigChoice>::names,
        },
    );
    pub const EV100: ConfigKey = ConfigKey::session("render.exposure.ev100", ConfigValueKind::NUMBER);
    pub const COMPENSATION: ConfigKey = ConfigKey::session("render.exposure.compensation", ConfigValueKind::NUMBER);

    pub fn read(values: &ConfigValues) -> ExposureDocument {
        ExposureDocument {
            mode: values.choice(ExposureDocument::MODE),
            ev100: values.number(ExposureDocument::EV100),
            compensation: values.number(ExposureDocument::COMPENSATION),
            adaptation: AdaptationDocument::read(values),
        }
    }

    pub fn into_config(self, defaults: &ExposureConfig) -> ExposureConfig {
        ExposureConfig {
            mode: self.mode.unwrap_or(defaults.mode),
            ev100: self.ev100.unwrap_or(defaults.ev100),
            compensation: self.compensation.unwrap_or(defaults.compensation),
            adaptation: self.adaptation.into_config(&defaults.adaptation),
        }
    }

    pub fn write(config: &ExposureConfig, layer: &mut ConfigLayer) {
        layer.set(ExposureDocument::MODE.path, config.mode.config_name().into());
        layer.set(ExposureDocument::EV100.path, ConfigValueKind::number(config.ev100));
        layer.set(
            ExposureDocument::COMPENSATION.path,
            ConfigValueKind::number(config.compensation),
        );
        AdaptationDocument::write(&config.adaptation, layer);
    }
}
