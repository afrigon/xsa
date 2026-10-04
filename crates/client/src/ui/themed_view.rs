use xui::{ModifiedContent, View};

use super::TextStyleModifier;

// The theme's modifiers on every view, like stylx's `View` extensions.
pub trait ThemedView: View + Sized {
    fn text_style(self, style: &str) -> ModifiedContent<Self, TextStyleModifier> {
        self.modifier(TextStyleModifier::new(style))
    }
}

impl<Content: View> ThemedView for Content {}
