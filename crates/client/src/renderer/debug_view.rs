// What a draw shows instead of its shaded color, from what every program has: its vertices and its material sample.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DebugView {
    Normals,
    Depth,
    Motion,
    Lighting,
    Triangles,
    Albedo,
    Roughness,
}

impl DebugView {
    // Matches `DebugView` in debug/view.slang, where 0 shows no view.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            DebugView::Normals => 1,
            DebugView::Depth => 2,
            DebugView::Motion => 3,
            DebugView::Lighting => 4,
            DebugView::Triangles => 5,
            DebugView::Albedo => 6,
            DebugView::Roughness => 7,
        }
    }
}
