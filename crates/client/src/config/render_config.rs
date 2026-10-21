use super::{AntialiasingConfig, BloomConfig, LensConfig, TerrainLodConfig, WhiteBalanceConfig};
use crate::renderer::{DebugView, SurfaceDebugView, TerrainDebugView, Tonemapper};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderConfig {
    pub tonemapper: Option<Tonemapper>,
    pub stars: bool,
    pub bloom: BloomConfig,
    pub lens: LensConfig,
    pub white_balance: WhiteBalanceConfig,
    pub antialiasing: AntialiasingConfig,
    pub wireframe: bool,
    pub shader_override: Option<DebugView>,
    pub terrain_shader_override: Option<TerrainDebugView>,
    pub surface_shader_override: Option<SurfaceDebugView>,
    pub terrain_lod: TerrainLodConfig,
}
