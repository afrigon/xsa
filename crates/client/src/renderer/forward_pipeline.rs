use super::Shader;
use super::debug_view::DebugView;
use super::geometry_variant::GeometryVariant;

pub(in crate::renderer) const NO_VIEW: u32 = 0;
// By constant id, as the programs declare them: the tile shape, the debug view, the terrain's, the shading range and
// the surface's view. A program ignores the ids it does not declare.
pub(in crate::renderer) const SPECIALIZATION_CONSTANTS: usize = 5;
const DEBUG_VIEW_CONSTANT: usize = 1;

// A program with the values its specialization constants are fixed to: one forward pipeline.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) struct ForwardPipeline {
    pub shader: Shader,
    pub view: Option<DebugView>,
    pub geometry: GeometryVariant,
}

impl ForwardPipeline {
    pub fn specialization_constants(self) -> [u32; SPECIALIZATION_CONSTANTS] {
        let mut constants = self.geometry.specialization_constants();
        constants[DEBUG_VIEW_CONSTANT] = self.view.map_or(NO_VIEW, DebugView::shader_id);
        constants
    }
}
