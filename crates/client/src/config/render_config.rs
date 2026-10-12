use super::{AntialiasingConfig, BloomConfig, LensConfig};
use crate::renderer::{Shader, ShadingModel, Tonemapper};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderConfig {
    pub tonemapper: Option<Tonemapper>,
    pub stars: bool,
    pub bloom: BloomConfig,
    pub lens: LensConfig,
    pub antialiasing: AntialiasingConfig,
    pub wireframe: bool,
    pub shader_override: Option<Shader>,
    pub shading_model: ShadingModel,
}
