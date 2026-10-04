use std::rc::Rc;

use xui::{Button, Context, Text, View};

use super::{Actions, ActionsKey, LinkButtonStyle, ThemedView};

// One entry of a menu: a link-style label that asks the game for something when clicked.
pub struct MenuButtonView {
    label: &'static str,
    action: Rc<dyn Fn(&Actions)>,
}

impl MenuButtonView {
    pub fn new(label: &'static str, action: impl Fn(&Actions) + 'static) -> MenuButtonView {
        MenuButtonView {
            label,
            action: Rc::new(action),
        }
    }
}

impl View for MenuButtonView {
    fn body(&self, context: &Context) -> impl View {
        let actions = context.environment().get::<ActionsKey>();
        let action = self.action.clone();

        Button::new(Text::new(self.label).text_style("menu"), move || {
            if let Some(actions) = &actions {
                action(actions);
            }
        })
        .button_style(LinkButtonStyle)
    }
}
