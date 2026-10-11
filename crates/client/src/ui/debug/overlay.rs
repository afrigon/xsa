use xui::{Context, EdgeInsets, Text, View};

use crate::ui::ThemedView;

const PADDING: f32 = 8.0;

pub struct DebugOverlayView {
    pub frames_per_second: Option<f64>,
    pub triangles: u64,
    pub ev100: f32,
    pub anchor_ev100: f32,
    pub star_visibility: f32,
}

impl View for DebugOverlayView {
    fn body(&self, _context: &Context) -> impl View {
        let frames_per_second = self
            .frames_per_second
            .map_or_else(|| "–".to_string(), |rate| format!("{rate:.0}"));

        Text::new(format!(
            "{frames_per_second} fps\n{} triangles\nEV {:.1} · anchor {:.1} · stars {:.2}",
            self.triangles, self.ev100, self.anchor_ev100, self.star_visibility
        ))
        .text_style("telemetry")
        .padding(EdgeInsets::all(PADDING))
    }
}
