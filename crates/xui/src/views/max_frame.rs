use crate::layout::MaxFrameLayout;
use crate::{Alignment, Context, Never, Node, SubviewEntry, UpdateContext, View};

pub struct MaxFrame<Content: View> {
    content: Content,
    layout: MaxFrameLayout,
}

impl<Content: View> MaxFrame<Content> {
    pub fn new(
        content: Content,
        max_width: Option<f32>,
        max_height: Option<f32>,
        alignment: Alignment,
    ) -> MaxFrame<Content> {
        MaxFrame {
            content,
            layout: MaxFrameLayout {
                max_width,
                max_height,
                alignment,
            },
        }
    }
}

impl<Content: View> View for MaxFrame<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.set_layout(self.layout);
        node.update_children(&[SubviewEntry::new(&self.content)], context)
    }
}
