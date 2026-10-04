use crate::{DrawList, EdgeInsets, Padding, Rect, Size, SizeProposal, ViewContext};

// Layout follows SwiftUI: a parent proposes a size, the view answers with the size it takes, and the parent
// places it. Sizes and positions are in logical points.
pub trait View {
    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size>;

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()>;

    fn padding(self, insets: EdgeInsets) -> Padding<Self>
    where
        Self: Sized,
    {
        Padding::new(self, insets)
    }
}
