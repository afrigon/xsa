use xsa_config::ConfigChoice;

use crate::renderer::SurfaceDebugView;

impl ConfigChoice for SurfaceDebugView {
    const ALL: &'static [Self] = &[
        SurfaceDebugView::Classes,
        SurfaceDebugView::Planes,
        SurfaceDebugView::Fade,
    ];

    fn config_name(self) -> &'static str {
        match self {
            SurfaceDebugView::Classes => "classes",
            SurfaceDebugView::Planes => "planes",
            SurfaceDebugView::Fade => "fade",
        }
    }
}
