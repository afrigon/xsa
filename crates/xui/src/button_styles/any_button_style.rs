use super::{ButtonConfiguration, ButtonStyle};
use crate::{AnyView, Context, ModifierContent};

// A button style behind one type, so the environment can carry any style.
pub trait AnyButtonStyle {
    fn any_body(&self, label: ModifierContent, configuration: ButtonConfiguration, context: &Context) -> AnyView;
}

impl<Style: ButtonStyle> AnyButtonStyle for Style {
    fn any_body(&self, label: ModifierContent, configuration: ButtonConfiguration, context: &Context) -> AnyView {
        AnyView::new(self.body(label, configuration, context))
    }
}
