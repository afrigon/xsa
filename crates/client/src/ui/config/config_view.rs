use xui::{Alignment, Context, DismissKey, EdgeInsets, HorizontalAlignment, Text, VStack, View};

use crate::ui::{MenuButtonView, ThemedView};

const LEADING_PADDING: f32 = 96.0;
const SPACING: f32 = 16.0;

// The game's settings, shown to players as Options.
pub struct ConfigView;

impl View for ConfigView {
    fn body(&self, context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);
        let dismiss = context.environment().get::<DismissKey>();

        VStack::new((
            Text::new("OPTIONS").text_style("title"),
            MenuButtonView::new("BACK", dismiss),
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
