use crate::{Alignment, DrawList, Environment, Never, Point, Rect, Size, SizeProposal, View, ViewContext};

pub struct FixedFrame<Content: View> {
    content: Content,
    width: Option<f32>,
    height: Option<f32>,
    alignment: Alignment,
}

impl<Content: View> FixedFrame<Content> {
    pub fn new(content: Content, width: Option<f32>, height: Option<f32>, alignment: Alignment) -> FixedFrame<Content> {
        FixedFrame {
            content,
            width,
            height,
            alignment,
        }
    }

    fn content_proposal(&self, proposal: SizeProposal) -> SizeProposal {
        SizeProposal {
            width: self.width.or(proposal.width),
            height: self.height.or(proposal.height),
        }
    }
}

impl<Content: View> View for FixedFrame<Content> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let content = self.content.size_that_fits(self.content_proposal(proposal), context)?;

        Ok(Size {
            width: self.width.unwrap_or(content.width),
            height: self.height.unwrap_or(content.height),
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
