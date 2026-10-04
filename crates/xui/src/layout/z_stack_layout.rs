use crate::{Alignment, DrawList, LayoutContext, Node, NodeLayout, Point, Rect, Size, SizeProposal};

// Overlays its children, later ones on top, each aligned within the stack's bounds.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct ZStackLayout {
    pub alignment: Alignment,
}

impl NodeLayout for ZStackLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        let mut context = context.with_stack_axis(None);
        let mut size = Size::default();

        for child in children {
            let child_size = child.size_that_fits(proposal, &mut context)?;
            size.width = size.width.max(child_size.width);
            size.height = size.height.max(child_size.height);
        }

        Ok(size)
    }

    fn place(
        &mut self,
        bounds: Rect,
        children: &mut [Node],
        context: &mut LayoutContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        let mut context = context.with_stack_axis(None);

        for child in children {
            let size = child.size_that_fits(SizeProposal::from(bounds.size), &mut context)?;
            let offset = self.alignment.position(bounds.size, size);
            let origin = Point {
                x: bounds.origin.x + offset.x,
                y: bounds.origin.y + offset.y,
            };
            child.place(Rect { origin, size }, &mut context, draw_list)?;
        }

        Ok(())
    }
}
