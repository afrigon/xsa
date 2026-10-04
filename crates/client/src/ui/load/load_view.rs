use xui::{Alignment, Context, DismissKey, EdgeInsets, HorizontalAlignment, Text, VStack, View};

use crate::ui::{MenuButtonView, ThemedView};

const LEADING_PADDING: f32 = 96.0;
const SPACING: f32 = 16.0;

pub struct LoadView;

impl View for LoadView {
    fn body(&self, context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);
        let dismiss = context.environment().get::<DismissKey>();

        VStack::new((
            Text::new("LOAD").text_style("title"),
            Text::new("No saves yet").text_style("telemetry"),
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
