use crate::{Alignment, DrawList, Environment, Never, Point, Rect, Size, SizeProposal, View, ViewContext};

// Overlays its subviews, later ones on top, each aligned within the stack's bounds.
pub struct ZStack<Content: View> {
    content: Content,
    alignment: Alignment,
}

impl<Content: View> ZStack<Content> {
    pub fn new(content: Content) -> ZStack<Content> {
        ZStack {
            content,
            alignment: Alignment::CENTER,
        }
    }

    pub fn alignment(self, alignment: Alignment) -> ZStack<Content> {
        ZStack { alignment, ..self }
    }
}

impl<Content: View> View for ZStack<Content> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let mut subviews = Vec::new();
        self.content.collect_subviews(&mut subviews);
        let mut context = context.with_stack_axis(None);
        let mut size = Size::default();

        for subview in subviews {
            let subview_size = subview.measure(proposal, &mut context)?;
            size.width = size.width.max(subview_size.width);
            size.height = size.height.max(subview_size.height);
        }

        Ok(size)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        self.content.collect_subviews(&mut subviews);
        let mut context = context.with_stack_axis(None);

        for subview in subviews {
            let size = subview.measure(SizeProposal::from(bounds.size), &mut context)?;
            let offset = self.alignment.position(bounds.size, size);
            let origin = Point {
                x: bounds.origin.x + offset.x,
                y: bounds.origin.y + offset.y,
            };
            subview.arrange(Rect { origin, size }, &mut context, draw_list)?;
        }

        Ok(())
    }
}
