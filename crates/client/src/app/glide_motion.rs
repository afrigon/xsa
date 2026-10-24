use glam::DVec2;
use xsa_input::ScrollDelta;

use super::App;

// The whole of what a glide injects while it holds its buttons; each update injects a share of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum GlideMotion {
    Hold,
    Mouse { offset: DVec2 },
    Scroll { amount: ScrollDelta },
}

impl GlideMotion {
    pub fn inject(&self, app: &mut App, share: f64) -> anyhow::Result<()> {
        match self {
            GlideMotion::Hold => Ok(()),
            GlideMotion::Mouse { offset } => app.inject_mouse_offset(*offset * share),
            GlideMotion::Scroll { amount } => app.inject_scroll(amount.scaled(share)),
        }
    }
}
