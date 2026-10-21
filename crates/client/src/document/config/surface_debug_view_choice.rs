use super::ConfigChoice;
use crate::renderer::SurfaceDebugView;

// No override leaves surfaced terrain draws to the terrain and generic overrides, or to their program.
impl ConfigChoice for Option<SurfaceDebugView> {
    const ALL: &'static [Self] = &[
        None,
        Some(SurfaceDebugView::Classes),
        Some(SurfaceDebugView::Planes),
        Some(SurfaceDebugView::Fade),
    ];

    fn config_name(self) -> &'static str {
        match self {
            None => "none",
            Some(SurfaceDebugView::Classes) => "classes",
            Some(SurfaceDebugView::Planes) => "planes",
            Some(SurfaceDebugView::Fade) => "fade",
        }
    }
}
