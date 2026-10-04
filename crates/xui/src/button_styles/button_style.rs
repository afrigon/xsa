use super::ButtonConfiguration;
use crate::{Context, ModifierContent, View};

// How buttons look, like SwiftUI's `ButtonStyle`: `body` builds on `label`, which stands for the button's label,
// for the button's current interaction state.
pub trait ButtonStyle: 'static {
    fn body(&self, label: ModifierContent, configuration: ButtonConfiguration, context: &Context) -> impl View;
}
