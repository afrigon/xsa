use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

// One of two views, for an `if`/`else` whose branches have different types.
pub enum Either<First: View, Second: View> {
    First(First),
    Second(Second),
}

impl<First: View, Second: View> View for Either<First, Second> {
    fn body(&self, _environment: &Environment) -> impl View {
        Never::primitive_body()
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        match self {
            Either::First(view) => view.size_that_fits(proposal, context),
            Either::Second(view) => view.size_that_fits(proposal, context),
        }
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        match self {
            Either::First(view) => view.place(bounds, context, draw_list),
            Either::Second(view) => view.place(bounds, context, draw_list),
        }
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
        match self {
            Either::First(view) => view.collect_subviews(subviews),
            Either::Second(view) => view.collect_subviews(subviews),
        }
    }
}
