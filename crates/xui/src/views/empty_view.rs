use crate::{Context, Never, Node, SubviewEntry, UpdateContext, View};

// Nothing: no size, nothing drawn, and no subview, so stacks give it no spacing.
pub struct EmptyView;

impl View for EmptyView {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.update_children(&[], context)
    }

    fn collect_subviews<'a>(&'a self, _subviews: &mut Vec<SubviewEntry<'a>>) {}
}
