use std::rc::Rc;

use crate::layout::ButtonLayout;
use crate::{
    ButtonConfiguration, ButtonStyleKey, Context, ModifierContent, Never, Node, SubviewEntry, UpdateContext, View,
};

// Runs `action` when clicked, drawn by the environment's button style, like SwiftUI's `Button`.
pub struct Button<Label: View> {
    label: Label,
    action: Rc<dyn Fn()>,
}

impl<Label: View> Button<Label> {
    pub fn new(label: Label, action: impl Fn() + 'static) -> Button<Label> {
        Button {
            label,
            action: Rc::new(action),
        }
    }
}

impl<Label: View> View for Button<Label> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let layout = node.layout_mut(|| ButtonLayout {
            action: self.action.clone(),
            configuration: ButtonConfiguration::default(),
        });
        layout.action = self.action.clone();
        let configuration = layout.configuration;
        let style = context.environment.get::<ButtonStyleKey>();
        let body = style.any_body(
            ModifierContent::new(),
            configuration,
            &node.context(&context.environment),
        );

        node.update_children(
            &[SubviewEntry::new(&body)],
            &mut context.with_modifier_content(Some(&self.label)),
        )
    }
}
