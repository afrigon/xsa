use super::{AntialiasingConfig, BloomConfig, ExposureConfig};
use crate::renderer::{Shader, ShadingModel, Tonemapper};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderConfig {
    pub tonemapper: Tonemapper,
    pub stars: bool,
    pub bloom: BloomConfig,
    pub exposure: ExposureConfig,
    pub antialiasing: AntialiasingConfig,
    pub wireframe: bool,
    pub shader_override: Option<Shader>,
    pub shading_model: ShadingModel,
}
