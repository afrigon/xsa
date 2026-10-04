use crate::layout::StackLayout;
use crate::{Context, Never, Node, SubviewEntry, UpdateContext, View};

// Views identified by their index: inserting or removing items shifts the nodes of the ones after, so lists that
// change belong in a ForEach with ids.
impl<Content: View> View for Vec<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        self.collect_subviews(&mut subviews);
        node.set_layout(StackLayout::list());
        node.update_children(&subviews, context)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        for view in self {
            view.collect_subviews(subviews);
        }
    }
}
