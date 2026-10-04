use crate::{DrawList, LayoutContext, Node, NodeLayout, Rect, Size, SizeProposal};

// The layout of views with a single child that takes their place: views with a body, and modifiers that only
// change the environment.
pub struct PassthroughLayout;

impl NodeLayout for PassthroughLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        match children.first_mut() {
            Some(child) => child.size_that_fits(proposal, context),
            None => Ok(Size::default()),
        }
    }

    fn place(
        &mut self,
        bounds: Rect,
        children: &mut [Node],
        context: &mut LayoutContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        match children.first_mut() {
            Some(child) => child.place(bounds, context, draw_list),
            None => Ok(()),
        }
    }
}
