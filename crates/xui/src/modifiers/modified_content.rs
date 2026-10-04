use crate::{
    Context, ModifierContent, Never, Node, PassthroughLayout, SubviewEntry, UpdateContext, View, ViewModifier,
};

pub struct ModifiedContent<Content: View, Modifier: ViewModifier> {
    content: Content,
    modifier: Modifier,
}

impl<Content: View, Modifier: ViewModifier> ModifiedContent<Content, Modifier> {
    pub fn new(content: Content, modifier: Modifier) -> ModifiedContent<Content, Modifier> {
        ModifiedContent { content, modifier }
    }
}

impl<Content: View, Modifier: ViewModifier> View for ModifiedContent<Content, Modifier> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let body = self
            .modifier
            .body(ModifierContent::new(), &node.context(&context.environment));
        node.layout_mut(|| PassthroughLayout);
        node.update_children(
            &[SubviewEntry::new(&body)],
            &mut context.with_modifier_content(Some(&self.content)),
        )
    }
}
