use crate::{DrawList, LayoutContext, Node, NodeLayout, Rect, Size, SizeProposal};

// Keeps its child in the tree and in layout, but can stop drawing it and stop pointer events reaching it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct VisibilityLayout {
    pub drawn: bool,
    pub hit_testable: bool,
}

impl NodeLayout for VisibilityLayout {
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
            Some(child) if self.drawn => child.place(bounds, context, draw_list),
            _ => Ok(()),
        }
    }

    fn hit_testable(&self) -> bool {
        self.drawn && self.hit_testable
    }
}
