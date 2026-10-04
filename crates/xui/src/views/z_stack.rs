use crate::layout::ZStackLayout;
use crate::{Alignment, Context, Never, Node, UpdateContext, View};

// Overlays its subviews, later ones on top, each aligned within the stack's bounds.
pub struct ZStack<Content: View> {
    content: Content,
    alignment: Alignment,
}

impl<Content: View> ZStack<Content> {
    pub fn new(content: Content) -> ZStack<Content> {
        ZStack {
            content,
            alignment: Alignment::CENTER,
        }
    }

    pub fn alignment(self, alignment: Alignment) -> ZStack<Content> {
        ZStack { alignment, ..self }
    }
}

impl<Content: View> View for ZStack<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        self.content.collect_subviews(&mut subviews);
        node.set_layout(ZStackLayout {
            alignment: self.alignment,
        });
        node.update_children(&subviews, context)
    }
}
