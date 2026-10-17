use xui::{Alignment, Context, Text, View};

use crate::ui::ThemedView;

pub struct LoadingView;

impl View for LoadingView {
    fn body(&self, _context: &Context) -> impl View {
        let fill = Some(f32::INFINITY);

        Text::new("LOADING")
            .text_style("title")
            .max_frame(fill, fill, Alignment::CENTER)
    }
}
