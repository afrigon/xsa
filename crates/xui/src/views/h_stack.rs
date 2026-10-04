use crate::layout::{Axis, DEFAULT_SPACING, StackLayout};
use crate::{Alignment, Context, Never, Node, UpdateContext, VerticalAlignment, View};

pub struct HStack<Content: View> {
    content: Content,
    layout: StackLayout,
}

impl<Content: View> HStack<Content> {
    pub fn new(content: Content) -> HStack<Content> {
        HStack {
            content,
            layout: StackLayout {
                axis: Axis::Horizontal,
                alignment: Alignment::CENTER,
                spacing: DEFAULT_SPACING,
            },
        }
    }

    pub fn alignment(mut self, alignment: VerticalAlignment) -> HStack<Content> {
        self.layout.alignment.vertical = alignment;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> HStack<Content> {
        self.layout.spacing = spacing;
        self
    }
}

impl<Content: View> View for HStack<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        self.content.collect_subviews(&mut subviews);
        node.set_layout(self.layout);
        node.update_children(&subviews, context)
    }
}
