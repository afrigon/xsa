use crate::{Alignment, DrawList, LayoutContext, Node, NodeLayout, Point, Rect, Size, SizeProposal};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct FixedFrameLayout {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub alignment: Alignment,
}

impl FixedFrameLayout {
    fn content_proposal(&self, proposal: SizeProposal) -> SizeProposal {
        SizeProposal {
            width: self.width.or(proposal.width),
            height: self.height.or(proposal.height),
        }
    }
}

impl NodeLayout for FixedFrameLayout {
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
            width: self.width.unwrap_or(content.width),
            height: self.height.unwrap_or(content.height),
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
