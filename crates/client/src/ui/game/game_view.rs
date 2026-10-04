use xui::{Alignment, Context, View};

use crate::ui::{HudTargetView, TargetNameKey};

// The game's HUD, over the 3D world.
pub struct GameView;

impl View for GameView {
    fn body(&self, context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);

        context
            .environment()
            .get::<TargetNameKey>()
            .map(|name| HudTargetView { name }.max_frame(fill, fill, Alignment::TOP))
    }
}
