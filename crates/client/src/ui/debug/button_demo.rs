use xui::{Button, Context, EdgeInsets, Text, View};

use crate::ui::{LinkButtonStyle, ThemedView};

const BOTTOM_PADDING: f32 = 48.0;

// A temporary button to try pointer input: it counts its clicks in its own state.
pub struct DebugButtonDemoView;

impl View for DebugButtonDemoView {
    fn body(&self, context: &Context) -> impl View {
        let clicks = context.state(|| 0_u32);
        let counter = clicks.clone();

        Button::new(
            Text::new(format!("CLICKED {} TIMES", clicks.get())).text_style("title"),
            move || counter.update(|clicks| *clicks += 1),
        )
        .button_style(LinkButtonStyle)
        .padding(EdgeInsets {
            bottom: BOTTOM_PADDING,
            ..EdgeInsets::default()
        })
    }
}
