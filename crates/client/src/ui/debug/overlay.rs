use xui::{Context, EdgeInsets, Text, View};

use crate::ui::ThemedView;

const PADDING: f32 = 8.0;

pub struct DebugOverlayView {
    pub frames_per_second: Option<f64>,
    pub triangles: u64,
}

impl View for DebugOverlayView {
    fn body(&self, _context: &Context) -> impl View {
        let frames_per_second = self
            .frames_per_second
            .map_or_else(|| "–".to_string(), |rate| format!("{rate:.0}"));

        Text::new(format!("{frames_per_second} fps\n{} triangles", self.triangles))
            .text_style("telemetry")
            .padding(EdgeInsets::all(PADDING))
    }
}
