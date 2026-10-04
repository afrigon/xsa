use crate::{Context, Never, Node, PassthroughLayout, Subview, SubviewEntry, UpdateContext, View};

// Any view behind one type, like SwiftUI's `AnyView`, for collections of views of different types.
pub struct AnyView {
    view: Box<dyn Subview>,
}

impl AnyView {
    pub fn new(view: impl View) -> AnyView {
        AnyView { view: Box::new(view) }
    }
}

impl View for AnyView {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.layout_mut(|| PassthroughLayout);
        node.update_children(&[SubviewEntry::new(self.view.as_ref())], context)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        self.view.collect_into(subviews);
    }
}
