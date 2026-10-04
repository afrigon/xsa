use xui::{Alignment, Context, EdgeInsets, HorizontalAlignment, VStack, View};

use super::MainMenuActions;
use crate::ui::MenuButtonView;

const LEADING_PADDING: f32 = 96.0;
const SPACING: f32 = 16.0;

pub struct MainMenuView {
    pub actions: MainMenuActions,
}

impl View for MainMenuView {
    fn body(&self, _context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);
        let actions = &self.actions;

        VStack::new((
            MenuButtonView::new("CONTINUE", actions.continue_game.clone()),
            MenuButtonView::new("LOAD", actions.load.clone()),
            MenuButtonView::new("NEW GAME", actions.new_game.clone()),
            MenuButtonView::new("OPTIONS", actions.options.clone()),
            MenuButtonView::new("QUIT", actions.quit.clone()),
        ))
        .alignment(HorizontalAlignment::Leading)
        .spacing(SPACING)
        .padding(EdgeInsets {
            leading: LEADING_PADDING,
            ..EdgeInsets::default()
        })
        .max_frame(fill, fill, Alignment::LEADING)
    }
}
