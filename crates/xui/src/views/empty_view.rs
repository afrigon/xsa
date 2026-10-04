use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

// Nothing: no size, nothing drawn, and no subview, so stacks give it no spacing.
pub struct EmptyView;

impl View for EmptyView {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, _proposal: SizeProposal, _context: &mut ViewContext) -> anyhow::Result<Size> {
        Ok(Size::default())
    }

    fn place(&self, _bounds: Rect, _context: &mut ViewContext, _draw_list: &mut DrawList) -> anyhow::Result<()> {
        Ok(())
    }

    fn collect_subviews<'a>(&'a self, _subviews: &mut Vec<&'a dyn Subview>) {}
}
