use super::ExposureMode;

#[derive(Clone, Debug, PartialEq)]
pub struct ExposureConfig {
    pub mode: ExposureMode,
    pub ev100: f32,
}
