use xsa_packs::Id;
use xui::{EdgeInsets, Environment, Text, View};

use crate::theme::Theme;

const THEME_NAMESPACE: &str = "base";
const TEXT_STYLE: &str = "telemetry";
const PADDING: f32 = 8.0;

pub struct StatisticsOverlay {
    pub frames_per_second: Option<f64>,
    pub triangles: u64,
}

impl StatisticsOverlay {
    pub fn view(&self, theme: &Theme, environment: &Environment) -> anyhow::Result<impl View + use<>> {
        let style = Id::parse(TEXT_STYLE, THEME_NAMESPACE)?;
        let frames_per_second = self
            .frames_per_second
            .map_or_else(|| "–".to_string(), |rate| format!("{rate:.0}"));

        Ok(
            Text::new(format!("{frames_per_second} fps\n{} triangles", self.triangles))
                .font(theme.font(&style)?)
                .foreground(theme.color(&theme.foreground().default, environment))
                .padding(EdgeInsets::all(PADDING)),
        )
    }
}
