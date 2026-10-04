use crate::layout::{Axis, DEFAULT_SPACING, StackLayout};
use crate::{Alignment, Context, HorizontalAlignment, Never, Node, UpdateContext, View};

pub struct VStack<Content: View> {
    content: Content,
    layout: StackLayout,
}

impl<Content: View> VStack<Content> {
    pub fn new(content: Content) -> VStack<Content> {
        VStack {
            content,
            layout: StackLayout {
                axis: Axis::Vertical,
                alignment: Alignment::CENTER,
                spacing: DEFAULT_SPACING,
            },
        }
    }

    pub fn alignment(mut self, alignment: HorizontalAlignment) -> VStack<Content> {
        self.layout.alignment.horizontal = alignment;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> VStack<Content> {
        self.layout.spacing = spacing;
        self
    }
}

impl<Content: View> View for VStack<Content> {
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
