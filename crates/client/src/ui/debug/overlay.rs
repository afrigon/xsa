use xui::{Context, EdgeInsets, Text, View};

use crate::renderer::LensLights;
use crate::ui::ThemedView;

const PADDING: f32 = 8.0;

pub struct DebugOverlayView {
    pub frames_per_second: Option<f64>,
    pub triangles: u64,
    pub ev100: f32,
    pub anchor_ev100: f32,
    pub star_visibility: f32,
    pub light: f32,
    pub lens_lights: LensLights,
    pub gpu_milliseconds: Option<f32>,
}

impl View for DebugOverlayView {
    fn body(&self, _context: &Context) -> impl View {
        let frames_per_second = self
            .frames_per_second
            .map_or_else(|| "–".to_string(), |rate| format!("{rate:.0}"));
        let gpu_milliseconds = self
            .gpu_milliseconds
            .map_or_else(|| "–".to_string(), |milliseconds| format!("{milliseconds:.2}"));

        Text::new(format!(
            "{frames_per_second} fps · GPU {gpu_milliseconds} ms\n{} triangles\nEV {:.1} · anchor {:.1} · stars {:.2} · light {:.3}\n\
             lens {} drawn · {} culled · {} behind",
            self.triangles,
            self.ev100,
            self.anchor_ev100,
            self.star_visibility,
            self.light,
            self.lens_lights.drawn,
            self.lens_lights.culled,
            self.lens_lights.behind
        ))
        .text_style("telemetry")
        .padding(EdgeInsets::all(PADDING))
    }
}
