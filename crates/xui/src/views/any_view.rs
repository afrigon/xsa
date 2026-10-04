use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

// Any view behind one type, like SwiftUI's `AnyView`, for collections of views of different types.
pub struct AnyView {
    view: Box<dyn Subview>,
}

impl AnyView {
    pub fn new(view: impl View + 'static) -> AnyView {
        AnyView { view: Box::new(view) }
    }
}

impl View for AnyView {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        self.view.measure(proposal, context)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        self.view.arrange(bounds, context, draw_list)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
        self.view.collect_into(subviews);
    }
}
