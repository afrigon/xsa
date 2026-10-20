// What a terrain draw with surfaces shows instead of its shaded color, from how its surfaces are chosen and sampled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SurfaceDebugView {
    Classes,
    Planes,
    Fade,
    HexCells,
}

impl SurfaceDebugView {
    // Matches `SurfaceDebugView` in terrain/surface/debug-view.slang, where 0 shows no view.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            SurfaceDebugView::Classes => 1,
            SurfaceDebugView::Planes => 2,
            SurfaceDebugView::Fade => 3,
            SurfaceDebugView::HexCells => 4,
        }
    }
}
