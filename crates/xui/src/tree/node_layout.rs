use std::any::Any;

use crate::{DrawList, LayoutContext, Node, PointerEvent, Rect, Size, SizeProposal};

// How a node sizes and places itself and its children, and reacts to the pointer. A primitive view keeps what its
// layout needs here, and updates it when the view's inputs change.
pub trait NodeLayout: Any {
    fn size_that_fits(
        &mut self,
        proposal: SizeProposal,
        children: &mut [Node],
        context: &mut LayoutContext,
    ) -> anyhow::Result<Size>;

    fn place(
        &mut self,
        bounds: Rect,
        children: &mut [Node],
        context: &mut LayoutContext,
        draw_list: &mut DrawList,
    ) -> anyhow::Result<()>;

    // `bounds` is where the node was last placed and `claimed` whether a view above already took the event.
    // Returns whether this layout takes it.
    fn handle_pointer(&mut self, _bounds: Rect, _event: &PointerEvent, _claimed: bool) -> bool {
        false
    }
}
