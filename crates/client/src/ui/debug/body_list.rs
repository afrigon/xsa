use xui::{Alignment, EdgeInsets, Environment, ForEach, HorizontalAlignment, Spacer, Text, VStack, View};

use crate::ui::ThemedView;

const PADDING: f32 = 8.0;

pub struct DebugBodyListView {
    pub planets: Vec<String>,
    pub moons: Vec<String>,
    pub show_moons: bool,
}

impl View for DebugBodyListView {
    fn body(&self, _environment: &Environment) -> impl View {
        VStack::new((
            Text::new("Bodies").text_style("title"),
            ForEach::with_id(
                &self.planets,
                |name| (*name).clone(),
                |name| Text::new(name.clone()).text_style("telemetry"),
            ),
            self.show_moons.then(|| {
                VStack::new((
                    Text::new("Moons").text_style("title"),
                    ForEach::with_id(
                        &self.moons,
                        |name| (*name).clone(),
                        |name| Text::new(name.clone()).text_style("telemetry"),
                    ),
                ))
                .alignment(HorizontalAlignment::Trailing)
            }),
            Spacer::new(),
        ))
        .alignment(HorizontalAlignment::Trailing)
        .padding(EdgeInsets::all(PADDING))
        .max_frame(Some(f32::INFINITY), None, Alignment::TRAILING)
    }
}
