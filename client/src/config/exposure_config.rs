use super::{AdaptationConfig, ExposureMode};

#[derive(Clone, Debug, PartialEq)]
pub struct ExposureConfig {
    pub mode: ExposureMode,
    pub ev100: f32,
    pub compensation: f32,
    pub adaptation: AdaptationConfig,
}
