use std::marker::PhantomData;

use crate::{DrawList, Environment, EnvironmentKey, Never, Rect, Size, SizeProposal, View, ViewContext};

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
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let environment = context.environment.with::<Key>(self.value.clone());
        self.content
            .size_that_fits(proposal, &mut context.with_environment(environment))
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        let environment = context.environment.with::<Key>(self.value.clone());
        self.content
            .place(bounds, &mut context.with_environment(environment), draw_list)
    }
}
