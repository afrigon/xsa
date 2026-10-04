use super::{BloomConfig, ExposureConfig};
use crate::renderer::Tonemapper;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderConfig {
    pub tonemapper: Tonemapper,
    pub stars: bool,
    pub bloom: BloomConfig,
    pub exposure: ExposureConfig,
}
