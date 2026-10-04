use xsa_commands::command::Screen;
use xui::{Alignment, Context, HorizontalAlignment, Text, VStack, View};

use crate::ui::{MenuButtonView, ThemedView};

const SPACING: f32 = 16.0;

pub struct PauseView;

impl View for PauseView {
    fn body(&self, _context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);

        VStack::new((
            Text::new("PAUSED").text_style("title"),
            MenuButtonView::new("RESUME", |actions| actions.pop()),
            MenuButtonView::new("OPTIONS", |actions| actions.push(Screen::Config)),
            MenuButtonView::new("MAIN MENU", |actions| actions.set(Screen::MainMenu)),
            MenuButtonView::new("QUIT", |actions| actions.exit()),
        ))
        .alignment(HorizontalAlignment::Center)
        .spacing(SPACING)
        .max_frame(fill, fill, Alignment::CENTER)
    }
}
