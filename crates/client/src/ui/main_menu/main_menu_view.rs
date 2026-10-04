use xsa_commands::command::Screen;
use xui::{Alignment, Context, EdgeInsets, HorizontalAlignment, VStack, View};

use crate::ui::MenuButtonView;

const LEADING_PADDING: f32 = 96.0;
const SPACING: f32 = 16.0;

pub struct MainMenuView;

impl View for MainMenuView {
    fn body(&self, _context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);

        VStack::new((
            MenuButtonView::new("CONTINUE", |actions| actions.set(Screen::Game)),
            MenuButtonView::new("LOAD", |actions| actions.push(Screen::Load)),
            MenuButtonView::new("NEW GAME", |actions| actions.set(Screen::Game)),
            MenuButtonView::new("OPTIONS", |actions| actions.push(Screen::Config)),
            MenuButtonView::new("QUIT", |actions| actions.exit()),
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
