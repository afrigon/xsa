use crate::{DrawList, LayoutContext, Node, NodeLayout, Rect, Size, SizeProposal};

// Takes the space offered along its stack's axis; outside a stack, the space offered on both axes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct SpacerLayout {
    pub min_length: f32,
}

impl NodeLayout for SpacerLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        _children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        let length = |proposed: Option<f32>| proposed.unwrap_or(self.min_length).max(self.min_length);
        let Some(axis) = context.stack_axis else {
            return Ok(Size {
                width: length(proposal.width),
                height: length(proposal.height),
            });
        };

        Ok(axis.size(length(axis.main_proposal(proposal)), 0.0))
    }

    fn place(
        &mut self,
        _bounds: Rect,
        _children: &mut [Node],
        _context: &mut LayoutContext,
        _draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
