use xui::{Context, EmptyView, View};

// The game's HUD, over the 3D world.
pub struct GameView;

impl View for GameView {
    fn body(&self, _context: &Context) -> impl View {
        EmptyView
    }
}
