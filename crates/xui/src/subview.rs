use crate::{DrawList, Rect, Size, SizeProposal, View, ViewContext};

// `View` in a form that can sit behind a reference to `dyn`, so containers can hold children of different types.
// Every view implements it.
pub trait Subview {
    fn measure(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size>;

    fn arrange(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()>;

    fn collect_into<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>);
}

impl<Content: View> Subview for Content {
    fn measure(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        self.size_that_fits(proposal, context)
    }

    fn arrange(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        self.place(bounds, context, draw_list)
    }

    fn collect_into<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
        self.collect_subviews(subviews);
    }
}
