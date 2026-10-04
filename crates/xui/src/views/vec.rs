use crate::layout::StackLayout;
use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

impl<Content: View> View for Vec<Content> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        StackLayout::list().size_of_content(self, proposal, context)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        StackLayout::list().place_content(self, bounds, context, draw_list)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
        for view in self {
            view.collect_subviews(subviews);
        }
    }
}
