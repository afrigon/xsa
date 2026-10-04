use crate::layout::{Axis, DEFAULT_SPACING, StackLayout};
use crate::{Alignment, DrawList, Environment, Never, Rect, Size, SizeProposal, VerticalAlignment, View, ViewContext};

pub struct HStack<Content: View> {
    content: Content,
    layout: StackLayout,
}

impl<Content: View> HStack<Content> {
    pub fn new(content: Content) -> HStack<Content> {
        HStack {
            content,
            layout: StackLayout {
                axis: Axis::Horizontal,
                alignment: Alignment::CENTER,
                spacing: DEFAULT_SPACING,
            },
        }
    }

    pub fn alignment(mut self, alignment: VerticalAlignment) -> HStack<Content> {
        self.layout.alignment.vertical = alignment;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> HStack<Content> {
        self.layout.spacing = spacing;
        self
    }
}

impl<Content: View> View for HStack<Content> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        self.layout.size_of_content(&self.content, proposal, context)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        self.layout.place_content(&self.content, bounds, context, draw_list)
    }
}
