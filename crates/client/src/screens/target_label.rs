use xsa_packs::Id;
use xui::{EdgeInsets, Environment, Text, View};

use crate::theme::Theme;

const THEME_NAMESPACE: &str = "base";
const TEXT_STYLE: &str = "title";
const TOP_PADDING: f32 = 24.0;

pub struct TargetLabel {
    pub name: String,
}

impl TargetLabel {
    pub fn view(&self, theme: &Theme, environment: &Environment) -> anyhow::Result<impl View + use<>> {
        let style = Id::parse(TEXT_STYLE, THEME_NAMESPACE)?;

        Ok(Text::new(self.name.clone())
            .font(theme.font(&style)?)
            .foreground(theme.color(&theme.foreground().default, environment))
            .padding(EdgeInsets::top(TOP_PADDING)))
    }
}
