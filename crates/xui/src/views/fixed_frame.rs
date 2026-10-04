use crate::layout::FixedFrameLayout;
use crate::{Alignment, Context, Never, Node, SubviewEntry, UpdateContext, View};

pub struct FixedFrame<Content: View> {
    content: Content,
    layout: FixedFrameLayout,
}

impl<Content: View> FixedFrame<Content> {
    pub fn new(content: Content, width: Option<f32>, height: Option<f32>, alignment: Alignment) -> FixedFrame<Content> {
        FixedFrame {
            content,
            layout: FixedFrameLayout {
                width,
                height,
                alignment,
            },
        }
    }
}

impl<Content: View> View for FixedFrame<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.set_layout(self.layout);
        node.update_children(&[SubviewEntry::new(&self.content)], context)
    }
}
