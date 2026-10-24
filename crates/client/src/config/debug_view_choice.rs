use xsa_config::ConfigChoice;

use crate::renderer::DebugView;

impl ConfigChoice for DebugView {
    const ALL: &'static [Self] = &[
        DebugView::Normals,
        DebugView::Depth,
        DebugView::Motion,
        DebugView::Lighting,
        DebugView::Triangles,
        DebugView::Albedo,
        DebugView::Roughness,
    ];

    fn config_name(self) -> &'static str {
        match self {
            DebugView::Normals => "normals",
            DebugView::Depth => "depth",
            DebugView::Motion => "motion",
            DebugView::Lighting => "lighting",
            DebugView::Triangles => "triangles",
            DebugView::Albedo => "albedo",
            DebugView::Roughness => "roughness",
        }
    }
}
