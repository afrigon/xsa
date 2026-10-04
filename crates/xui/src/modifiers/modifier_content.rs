use crate::{Context, Never, Node, PassthroughLayout, SubviewEntry, UpdateContext, View};

// Stands for the view a modifier wraps, like SwiftUI's `ViewModifier.Content`: wherever a modifier's body places
// it, the wrapped view appears. Only xui creates it.
#[non_exhaustive]
pub struct ModifierContent {}

impl ModifierContent {
    pub(crate) fn new() -> ModifierContent {
        ModifierContent {}
    }
}

impl View for ModifierContent {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.layout_mut(|| PassthroughLayout);
        let Some(content) = context.modifier_content else {
            context.warn_once("a modifier's content was used outside of its modifier".to_string());
            return node.update_children(&[], context);
        };

        node.update_children(&[SubviewEntry::new(content)], &mut context.with_modifier_content(None))
    }
}
