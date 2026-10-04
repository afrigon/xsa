use crate::{Context, Never, Node, OpacityKey, PassthroughLayout, SubviewEntry, UpdateContext, View};

// Multiplies the opacity of its content and everything inside it.
pub struct OpacityModifier<Content: View> {
    content: Content,
    opacity: f32,
}

impl<Content: View> OpacityModifier<Content> {
    pub fn new(content: Content, opacity: f32) -> OpacityModifier<Content> {
        OpacityModifier { content, opacity }
    }
}

impl<Content: View> View for OpacityModifier<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let opacity = context.environment.get::<OpacityKey>() * self.opacity;
        let environment = context.environment.with::<OpacityKey>(opacity);
        node.layout_mut(|| PassthroughLayout);
        node.update_children(
            &[SubviewEntry::new(&self.content)],
            &mut context.with_environment(environment),
        )
    }
}
