use crate::layout::PaddingLayout;
use crate::{Context, EdgeInsets, Never, Node, SubviewEntry, UpdateContext, View};

pub struct Padding<Content: View> {
    content: Content,
    insets: EdgeInsets,
}

impl<Content: View> Padding<Content> {
    pub fn new(content: Content, insets: EdgeInsets) -> Padding<Content> {
        Padding { content, insets }
    }
}

impl<Content: View> View for Padding<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.set_layout(PaddingLayout { insets: self.insets });
        node.update_children(&[SubviewEntry::new(&self.content)], context)
    }
}
