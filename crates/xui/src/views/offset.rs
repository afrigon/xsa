use crate::layout::OffsetLayout;
use crate::{Context, Never, Node, Point, SubviewEntry, UpdateContext, View};

pub struct Offset<Content: View> {
    content: Content,
    offset: Point,
}

impl<Content: View> Offset<Content> {
    pub fn new(content: Content, offset: Point) -> Offset<Content> {
        Offset { content, offset }
    }
}

impl<Content: View> View for Offset<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.set_layout(OffsetLayout { offset: self.offset });
        node.update_children(&[SubviewEntry::new(&self.content)], context)
    }
}
