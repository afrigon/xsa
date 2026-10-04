use std::marker::PhantomData;

use crate::{Context, EnvironmentKey, Never, Node, PassthroughLayout, SubviewEntry, UpdateContext, View};

// Sets one environment value for its content and everything inside it.
pub struct EnvironmentModifier<Content: View, Key: EnvironmentKey> {
    content: Content,
    value: Key::Value,
    key: PhantomData<Key>,
}

impl<Content: View, Key: EnvironmentKey> EnvironmentModifier<Content, Key> {
    pub fn new(content: Content, value: Key::Value) -> EnvironmentModifier<Content, Key> {
        EnvironmentModifier {
            content,
            value,
            key: PhantomData,
        }
    }
}

impl<Content: View, Key: EnvironmentKey> View for EnvironmentModifier<Content, Key> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let environment = context.environment.with::<Key>(self.value.clone());
        node.layout_mut(|| PassthroughLayout);
        node.update_children(
            &[SubviewEntry::new(&self.content)],
            &mut context.with_environment(environment),
        )
    }
}
