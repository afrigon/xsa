use crate::layout::StackLayout;
use crate::{Context, Never, Node, SubviewEntry, UpdateContext, View};

// An optional view, the result of an `if` without `else`: `None` is no subview at all.
impl<Content: View> View for Option<Content> {
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
        if let Some(view) = self {
            view.collect_subviews(subviews);
        }
    }
}
