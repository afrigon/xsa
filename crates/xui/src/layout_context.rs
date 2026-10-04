use crate::atlas::GlyphAtlas;
use crate::layout::Axis;

// What the layout pass carries down the tree.
pub struct LayoutContext<'a> {
    pub(crate) atlas: &'a mut GlyphAtlas,
    pub(crate) stack_axis: Option<Axis>,
}

impl LayoutContext<'_> {
    pub(crate) fn with_stack_axis(&mut self, stack_axis: Option<Axis>) -> LayoutContext<'_> {
        LayoutContext {
            atlas: self.atlas,
            stack_axis,
        }
    }
}
