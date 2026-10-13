use super::{AntialiasingConfig, BloomConfig, EyeAdaptationConfig, LensConfig, TerrainLodConfig, WhiteBalanceConfig};
use crate::renderer::{Shader, ShadingModel, Tonemapper};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderConfig {
    pub tonemapper: Option<Tonemapper>,
    pub stars: bool,
    pub bloom: BloomConfig,
    pub lens: LensConfig,
    pub white_balance: WhiteBalanceConfig,
    pub eye_adaptation: EyeAdaptationConfig,
    pub antialiasing: AntialiasingConfig,
    pub wireframe: bool,
    pub shader_override: Option<Shader>,
    pub shading_model: ShadingModel,
    pub terrain_lod: TerrainLodConfig,
}
