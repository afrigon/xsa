use std::sync::LazyLock;

use xsa_packs::Id;
use xui::{ColorSchemeKey, Context, EdgeInsets, ForegroundStyleKey, Text, View};

use crate::input::InputSource;
use crate::theme::ThemeKey;
use crate::ui::ThemedView;

const PADDING: f32 = 8.0;

static WARNING_ROLE: LazyLock<Id> =
    LazyLock::new(|| Id::parse("critical", "base").expect("the critical role id is valid"));

pub struct InputDisabledView {
    pub sources: Vec<InputSource>,
}

impl View for InputDisabledView {
    fn body(&self, context: &Context) -> impl View {
        let environment = context.environment();
        let color = environment
            .get::<ThemeKey>()
            .and_then(|theme| {
                let role = theme.color_role(&WARNING_ROLE)?;
                Some(theme.color(&role.emphasis, environment.get::<ColorSchemeKey>()))
            })
            .unwrap_or_else(|| environment.get::<ForegroundStyleKey>());

        let label = match self.sources.as_slice() {
            [InputSource::Raw] => "player input disabled",
            [InputSource::Command] => "command input disabled",
            _ => "input disabled",
        };

        Text::new(label)
            .text_style("telemetry")
            .foreground_style(color)
            .padding(EdgeInsets::all(PADDING))
    }
}
