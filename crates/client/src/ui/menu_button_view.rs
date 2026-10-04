use xui::{Action, Button, Context, Text, View};

use super::{LinkButtonStyle, ThemedView};

// One entry of a menu: a link-style label that performs its action when clicked.
pub struct MenuButtonView {
    label: &'static str,
    action: Action,
}

impl MenuButtonView {
    pub fn new(label: &'static str, action: Action) -> MenuButtonView {
        MenuButtonView { label, action }
    }
}

impl View for MenuButtonView {
    fn body(&self, _context: &Context) -> impl View {
        let action = self.action.clone();

        Button::new(Text::new(self.label).text_style("menu"), move || action.perform()).button_style(LinkButtonStyle)
    }
}
