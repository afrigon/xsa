use crate::{DrawList, LayoutContext, Node, NodeLayout, Point, Rect, Size, SizeProposal};

// Draws the child shifted, keeping the size it takes in its parent's layout.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct OffsetLayout {
    pub offset: Point,
}

impl NodeLayout for OffsetLayout {
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
        let Some(child) = children.first_mut() else {
            return Ok(());
        };
        let shifted = Rect {
            origin: Point {
                x: bounds.origin.x + self.offset.x,
                y: bounds.origin.y + self.offset.y,
            },
            size: bounds.size,
        };

        child.place(shifted, context, draw_list)
    }
}
