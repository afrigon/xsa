use crate::layout::{Axis, DEFAULT_SPACING, StackLayout};
use crate::{
    Alignment, DrawList, Environment, HorizontalAlignment, Never, Rect, Size, SizeProposal, View, ViewContext,
};

pub struct VStack<Content: View> {
    content: Content,
    layout: StackLayout,
}

impl<Content: View> VStack<Content> {
    pub fn new(content: Content) -> VStack<Content> {
        VStack {
            content,
            layout: StackLayout {
                axis: Axis::Vertical,
                alignment: Alignment::CENTER,
                spacing: DEFAULT_SPACING,
            },
        }
    }

    pub fn alignment(mut self, alignment: HorizontalAlignment) -> VStack<Content> {
        self.layout.alignment.horizontal = alignment;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> VStack<Content> {
        self.layout.spacing = spacing;
        self
    }
}

impl<Content: View> View for VStack<Content> {
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
