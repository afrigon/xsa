use xui::{Alignment, Context, DismissKey, HorizontalAlignment, Text, VStack, View};

use super::PauseActions;
use crate::ui::{MenuButtonView, ThemedView};

const SPACING: f32 = 16.0;

pub struct PauseView {
    pub actions: PauseActions,
}

impl View for PauseView {
    fn body(&self, context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);
        let dismiss = context.environment().get::<DismissKey>();

        VStack::new((
            Text::new("PAUSED").text_style("title"),
            MenuButtonView::new("RESUME", dismiss),
            MenuButtonView::new("OPTIONS", self.actions.options.clone()),
            MenuButtonView::new("MAIN MENU", self.actions.main_menu.clone()),
        ))
        .alignment(HorizontalAlignment::Center)
        .spacing(SPACING)
        .max_frame(fill, fill, Alignment::CENTER)
    }
}
