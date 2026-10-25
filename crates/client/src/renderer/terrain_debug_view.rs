// What a terrain draw shows instead of its shaded color, from its tiles and their pages.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TerrainDebugView {
    Lod,
    Scale,
    PageLevels,
    PageFallback,
}

impl TerrainDebugView {
    // Matches `TerrainDebugView` in terrain/debug/terrain-view.slang, where 0 shows no view.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            TerrainDebugView::Lod => 1,
            TerrainDebugView::Scale => 2,
            TerrainDebugView::PageLevels => 3,
            TerrainDebugView::PageFallback => 4,
        }
    }
}
