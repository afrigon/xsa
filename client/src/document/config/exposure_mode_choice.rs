use super::ConfigChoice;
use crate::config::ExposureMode;

impl ConfigChoice for ExposureMode {
    const ALL: &'static [Self] = &[ExposureMode::Manual, ExposureMode::EyeAdaptation];

    fn config_name(self) -> &'static str {
        match self {
            ExposureMode::Manual => "manual",
            ExposureMode::EyeAdaptation => "eye-adaptation",
        }
    }
}
