use super::{ButtonConfiguration, ButtonStyle};
use crate::{Context, ModifierContent, View};

// The label as it is, whatever the interaction state.
pub struct PlainButtonStyle;

impl ButtonStyle for PlainButtonStyle {
    fn body(&self, label: ModifierContent, _configuration: ButtonConfiguration, _context: &Context) -> impl View {
        label
    }
}
