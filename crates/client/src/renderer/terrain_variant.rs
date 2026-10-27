use super::forward_pipeline::{NO_VIEW, SPECIALIZATION_CONSTANTS};
use super::shading_range::ShadingRange;
use super::surface_debug_view::SurfaceDebugView;
use super::terrain_debug_view::TerrainDebugView;
use super::tile_shape::TileShape;

// A terrain pipeline's tile shape, shading range and views; programs without surfaces do not declare the surface's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) struct TerrainVariant {
    pub shape: TileShape,
    pub range: ShadingRange,
    pub terrain_view: Option<TerrainDebugView>,
    pub surface_view: Option<SurfaceDebugView>,
}

impl TerrainVariant {
    pub fn specialization_constants(self) -> [u32; SPECIALIZATION_CONSTANTS] {
        [
            self.shape.shader_id(),
            NO_VIEW,
            self.terrain_view.map_or(NO_VIEW, TerrainDebugView::shader_id),
            self.range.shader_id(),
            self.surface_view.map_or(NO_VIEW, SurfaceDebugView::shader_id),
        ]
    }
}
