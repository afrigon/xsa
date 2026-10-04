use crate::{Alignment, DrawList, Environment, Never, Point, Rect, Size, SizeProposal, View, ViewContext};

pub struct MaxFrame<Content: View> {
    content: Content,
    max_width: Option<f32>,
    max_height: Option<f32>,
    alignment: Alignment,
}

impl<Content: View> MaxFrame<Content> {
    pub fn new(
        content: Content,
        max_width: Option<f32>,
        max_height: Option<f32>,
        alignment: Alignment,
    ) -> MaxFrame<Content> {
        MaxFrame {
            content,
            max_width,
            max_height,
            alignment,
        }
    }

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

impl<Content: View> View for MaxFrame<Content> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let content = self.content.size_that_fits(self.content_proposal(proposal), context)?;

        Ok(Size {
            width: MaxFrame::<Content>::length(proposal.width, self.max_width, content.width),
            height: MaxFrame::<Content>::length(proposal.height, self.max_height, content.height),
        })
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        let size = self
            .content
            .size_that_fits(self.content_proposal(SizeProposal::from(bounds.size)), context)?;
        let offset = self.alignment.position(bounds.size, size);
        let origin = Point {
            x: bounds.origin.x + offset.x,
            y: bounds.origin.y + offset.y,
        };

        self.content.place(Rect { origin, size }, context, draw_list)
    }
}
