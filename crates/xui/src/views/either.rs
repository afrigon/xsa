use crate::{Context, Never, Node, PassthroughLayout, SubviewEntry, UpdateContext, View};

// One of two views, for an `if`/`else` whose branches have different types. Switching branch is a different view,
// with a new node, as in SwiftUI.
pub enum Either<First: View, Second: View> {
    First(First),
    Second(Second),
}

impl<First: View, Second: View> View for Either<First, Second> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let branch = match self {
            Either::First(view) => SubviewEntry::new(view),
            Either::Second(view) => SubviewEntry::new(view),
        };
        node.layout_mut(|| PassthroughLayout);
        node.update_children(&[branch], context)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        match self {
            Either::First(view) => view.collect_subviews(subviews),
            Either::Second(view) => view.collect_subviews(subviews),
        }
    }
}
