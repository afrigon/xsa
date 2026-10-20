// What a terrain draw with surfaces shows instead of its shaded color, from how its surfaces are chosen and sampled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SurfaceDebugView {
    Classes,
    Scales,
    Planes,
    Fade,
}

impl SurfaceDebugView {
    // Matches `SurfaceDebugView` in terrain/surface/debug-view.slang, where 0 shows no view.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            SurfaceDebugView::Classes => 1,
            SurfaceDebugView::Scales => 2,
            SurfaceDebugView::Planes => 3,
            SurfaceDebugView::Fade => 4,
        }
    }
}
