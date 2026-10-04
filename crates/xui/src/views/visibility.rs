use crate::layout::VisibilityLayout;
use crate::{Context, Never, Node, SubviewEntry, UpdateContext, View};

// Whether its content is drawn and receives pointer events; either way it keeps its state.
pub struct Visibility<Content: View> {
    content: Content,
    layout: VisibilityLayout,
}

impl<Content: View> Visibility<Content> {
    pub fn new(content: Content, drawn: bool, hit_testable: bool) -> Visibility<Content> {
        Visibility {
            content,
            layout: VisibilityLayout { drawn, hit_testable },
        }
    }
}

impl<Content: View> View for Visibility<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.set_layout(self.layout);
        node.update_children(&[SubviewEntry::new(&self.content)], context)
    }
}
