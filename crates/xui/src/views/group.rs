use crate::layout::StackLayout;
use crate::{Context, Never, Node, SubviewEntry, UpdateContext, View};

// Gathers views into one without laying them out itself: a stack around it lays out the group's items as its own.
pub struct Group<Content: View> {
    content: Content,
}

impl<Content: View> Group<Content> {
    pub fn new(content: Content) -> Group<Content> {
        Group { content }
    }
}

impl<Content: View> View for Group<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        self.content.collect_subviews(&mut subviews);
        node.set_layout(StackLayout::list());
        node.update_children(&subviews, context)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        self.content.collect_subviews(subviews);
    }
}
