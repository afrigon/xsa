use super::ConfigChoice;
use crate::renderer::DebugView;

// No override shades every draw with its program.
impl ConfigChoice for Option<DebugView> {
    const ALL: &'static [Self] = &[
        None,
        Some(DebugView::Normals),
        Some(DebugView::Depth),
        Some(DebugView::Motion),
        Some(DebugView::Lighting),
        Some(DebugView::Triangles),
        Some(DebugView::Albedo),
        Some(DebugView::Roughness),
    ];

    fn config_name(self) -> &'static str {
        match self {
            None => "none",
            Some(DebugView::Normals) => "normals",
            Some(DebugView::Depth) => "depth",
            Some(DebugView::Motion) => "motion",
            Some(DebugView::Lighting) => "lighting",
            Some(DebugView::Triangles) => "triangles",
            Some(DebugView::Albedo) => "albedo",
            Some(DebugView::Roughness) => "roughness",
        }
    }
}
