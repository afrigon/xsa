use std::rc::Rc;

use crate::{
    ButtonConfiguration, DrawList, LayoutContext, Node, NodeLayout, PointerButton, PointerEvent, PointerEventKind,
    Rect, Size, SizeProposal,
};

// A button's interaction state and current action. The action fires on a primary release inside the button
// after a press that started in it, as on iOS: releasing outside cancels.
pub(crate) struct ButtonLayout {
    pub action: Rc<dyn Fn()>,
    pub configuration: ButtonConfiguration,
}

impl NodeLayout for ButtonLayout {
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
        match children.first_mut() {
            Some(child) => child.place(bounds, context, draw_list),
            None => Ok(()),
        }
    }

    fn handle_pointer(&mut self, bounds: Rect, event: &PointerEvent, claimed: bool) -> bool {
        let under_pointer = !claimed && bounds.contains(event.position);

        match event.kind {
            PointerEventKind::Moved => {
                self.configuration.is_hovered = under_pointer;
                under_pointer
            }
            PointerEventKind::Left => {
                self.configuration.is_hovered = false;
                false
            }
            PointerEventKind::Pressed {
                button: PointerButton::Primary,
            } => {
                self.configuration.is_pressed = under_pointer;
                under_pointer
            }
            PointerEventKind::Released {
                button: PointerButton::Primary,
            } if self.configuration.is_pressed => {
                self.configuration.is_pressed = false;

                if under_pointer {
                    (self.action)();
                }

                true
            }
            PointerEventKind::Pressed { .. } | PointerEventKind::Released { .. } => false,
        }
    }
}
