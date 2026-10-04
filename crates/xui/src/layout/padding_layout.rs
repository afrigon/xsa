use crate::{DrawList, EdgeInsets, LayoutContext, Node, NodeLayout, Rect, Size, SizeProposal};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct PaddingLayout {
    pub insets: EdgeInsets,
}

impl NodeLayout for PaddingLayout {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        let content = match children.first_mut() {
            Some(child) => child.size_that_fits(proposal.inset(self.insets), context)?,
            None => Size::default(),
        };

        Ok(Size {
            width: content.width + self.insets.horizontal(),
            height: content.height + self.insets.vertical(),
        })
    }

    fn place(
        &mut self,
        bounds: Rect,
        children: &mut [Node],
        context: &mut LayoutContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        match children.first_mut() {
            Some(child) => child.place(bounds.inset(self.insets), context, draw_list),
            None => Ok(()),
        }
    }
}
