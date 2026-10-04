use xui::{EdgeInsets, Environment, Text, View};

use crate::ui::ThemedView;

const TOP_PADDING: f32 = 24.0;

pub struct HudTargetView {
    pub name: String,
}

impl View for HudTargetView {
    fn body(&self, _environment: &Environment) -> impl View {
        Text::new(self.name.clone())
            .text_style("title")
            .padding(EdgeInsets::top(TOP_PADDING))
    }
}
