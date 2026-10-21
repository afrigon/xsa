use super::ConfigChoice;
use crate::renderer::TerrainDebugView;

// No override leaves terrain draws to the generic override, or to their program.
impl ConfigChoice for Option<TerrainDebugView> {
    const ALL: &'static [Self] = &[
        None,
        Some(TerrainDebugView::Lod),
        Some(TerrainDebugView::Scale),
        Some(TerrainDebugView::PageLevels),
        Some(TerrainDebugView::PageFallback),
    ];

    fn config_name(self) -> &'static str {
        match self {
            None => "none",
            Some(TerrainDebugView::Lod) => "lod",
            Some(TerrainDebugView::Scale) => "scale",
            Some(TerrainDebugView::PageLevels) => "page-levels",
            Some(TerrainDebugView::PageFallback) => "page-fallback",
        }
    }
}
