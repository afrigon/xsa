use crate::{Alignment, DrawList, LayoutContext, Node, NodeLayout, Point, Rect, Size, SizeProposal};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct MaxFrameLayout {
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub alignment: Alignment,
}

impl MaxFrameLayout {
    fn content_proposal(&self, proposal: SizeProposal) -> SizeProposal {
        let limit = |proposed: Option<f32>, maximum: Option<f32>| {
            proposed.map(|proposed| maximum.map_or(proposed, |maximum| proposed.min(maximum)))
        };

        SizeProposal {
            width: limit(proposal.width, self.max_width),
            height: limit(proposal.height, self.max_height),
        }
    }

    // With a maximum, the frame takes the proposed length up to it, but never less than its content.
    fn length(proposed: Option<f32>, maximum: Option<f32>, content: f32) -> f32 {
        match (proposed, maximum) {
            (Some(proposed), Some(maximum)) => proposed.min(maximum).max(content),
            _ => content,
        }
    }
}

impl NodeLayout for MaxFrameLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        let content = match children.first_mut() {
            Some(child) => child.size_that_fits(self.content_proposal(proposal), context)?,
            None => Size::default(),
        };

        Ok(Size {
            width: MaxFrameLayout::length(proposal.width, self.max_width, content.width),
            height: MaxFrameLayout::length(proposal.height, self.max_height, content.height),
        })
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
        let size = child.size_that_fits(self.content_proposal(SizeProposal::from(bounds.size)), context)?;
        let offset = self.alignment.position(bounds.size, size);
        let origin = Point {
            x: bounds.origin.x + offset.x,
            y: bounds.origin.y + offset.y,
        };

        child.place(Rect { origin, size }, context, draw_list)
    }
}
