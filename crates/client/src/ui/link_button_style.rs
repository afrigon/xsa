use std::sync::LazyLock;

use xsa_packs::Id;
use xui::{ButtonConfiguration, ButtonStyle, ColorSchemeKey, Context, ForegroundStyleKey, ModifierContent, View};

use crate::theme::ThemeKey;

static HIGHLIGHT_ROLE: LazyLock<Id> =
    LazyLock::new(|| Id::parse("primary", "base").expect("the primary role id is valid"));

// Buttons that look like links: the label alone, in the inherited foreground, turning the primary color while
// hovered or pressed.
pub struct LinkButtonStyle;

impl ButtonStyle for LinkButtonStyle {
    fn body(&self, label: ModifierContent, configuration: ButtonConfiguration, context: &Context) -> impl View {
        let environment = context.environment();
        let highlighted = configuration.is_hovered || configuration.is_pressed;
        let highlight = environment.get::<ThemeKey>().and_then(|theme| {
            let role = theme.color_role(&HIGHLIGHT_ROLE)?;
            Some(theme.color(&role.emphasis, environment.get::<ColorSchemeKey>()))
        });
        let color = match highlight {
            Some(highlight) if highlighted => highlight,
            _ => environment.get::<ForegroundStyleKey>(),
        };

        label.foreground_style(color)
    }
}
