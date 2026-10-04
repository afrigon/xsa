use xsa_packs::Id;
use xui::{Context, FontKey, ModifierContent, View, ViewModifier};

use crate::theme::ThemeKey;

const THEME_NAMESPACE: &str = "base";

// Sets the font of a theme text style; without a theme or a matching style the inherited font stays.
pub struct TextStyleModifier {
    style: Option<Id>,
}

impl TextStyleModifier {
    pub fn new(style: &str) -> TextStyleModifier {
        TextStyleModifier {
            style: Id::parse(style, THEME_NAMESPACE).ok(),
        }
    }
}

impl ViewModifier for TextStyleModifier {
    fn body(&self, content: ModifierContent, context: &Context) -> impl View {
        let environment = context.environment();
        let font = environment
            .get::<ThemeKey>()
            .zip(self.style.as_ref())
            .and_then(|(theme, style)| theme.font(style))
            .or_else(|| environment.get::<FontKey>());

        content.font(font)
    }
}
