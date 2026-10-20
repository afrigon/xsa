use super::Shader;
use super::debug_view::DebugView;
use super::shading_range::ShadingRange;
use super::terrain_debug_view::TerrainDebugView;
use super::tile_shape::TileShape;

const NO_VIEW: u32 = 0;

// A terrain program with the values its specialization constants are fixed to: one forward pipeline.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) struct ForwardPipeline {
    pub shader: Shader,
    pub shape: TileShape,
    pub range: ShadingRange,
    pub view: Option<DebugView>,
    pub terrain_view: Option<TerrainDebugView>,
}

impl ForwardPipeline {
    // By constant id, as the programs declare them: the tile shape, the debug view, the terrain's, then the shading
    // range, which programs without surfaces do not declare.
    pub fn specialization_constants(self) -> [u32; 4] {
        [
            self.shape.shader_id(),
            self.view.map_or(NO_VIEW, DebugView::shader_id),
            self.terrain_view.map_or(NO_VIEW, TerrainDebugView::shader_id),
            self.range.shader_id(),
        ]
    }
}
