use crate::layout::StackLayout;
use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

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
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        StackLayout::list().size_of_content(&self.content, proposal, context)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        StackLayout::list().place_content(&self.content, bounds, context, draw_list)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
        self.content.collect_subviews(subviews);
    }
}
