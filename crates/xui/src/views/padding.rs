use crate::{DrawList, EdgeInsets, Environment, Never, Rect, Size, SizeProposal, View, ViewContext};

pub struct Padding<Content: View> {
    content: Content,
    insets: EdgeInsets,
}

impl<Content: View> Padding<Content> {
    pub fn new(content: Content, insets: EdgeInsets) -> Padding<Content> {
        Padding { content, insets }
    }
}

impl<Content: View> View for Padding<Content> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let content = self.content.size_that_fits(proposal.inset(self.insets), context)?;

        Ok(Size {
            width: content.width + self.insets.horizontal(),
            height: content.height + self.insets.vertical(),
        })
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        self.content.place(bounds.inset(self.insets), context, draw_list)
    }
}
