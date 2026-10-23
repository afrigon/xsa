use xsa_config::ConfigChoice;

use crate::renderer::TerrainDebugView;

impl ConfigChoice for TerrainDebugView {
    const ALL: &'static [Self] = &[
        TerrainDebugView::Lod,
        TerrainDebugView::Scale,
        TerrainDebugView::PageLevels,
        TerrainDebugView::PageFallback,
    ];

    fn config_name(self) -> &'static str {
        match self {
            TerrainDebugView::Lod => "lod",
            TerrainDebugView::Scale => "scale",
            TerrainDebugView::PageLevels => "page-levels",
            TerrainDebugView::PageFallback => "page-fallback",
        }
    }
}
