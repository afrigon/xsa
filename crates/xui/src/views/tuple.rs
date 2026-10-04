use crate::layout::StackLayout;
use crate::{DrawList, Environment, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

// A tuple of views is a list of them, like SwiftUI's `TupleView`: a stack lays out each member's subviews.
macro_rules! tuple_view {
    ($($member:ident),+) => {
        impl<$($member: View),+> View for ($($member,)+) {
            fn body(&self, _environment: &Environment) -> impl View {
                Never::primitive_body()
            }

            fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
                StackLayout::list().size_of_content(self, proposal, context)
            }

            fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
                StackLayout::list().place_content(self, bounds, context, draw_list)
            }

            #[allow(non_snake_case)]
            fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
                let ($($member,)+) = self;
                $($member.collect_subviews(subviews);)+
            }
        }
    };
}

tuple_view!(A);
tuple_view!(A, B);
tuple_view!(A, B, C);
tuple_view!(A, B, C, D);
tuple_view!(A, B, C, D, E);
tuple_view!(A, B, C, D, E, F);
tuple_view!(A, B, C, D, E, F, G);
tuple_view!(A, B, C, D, E, F, G, H);
tuple_view!(A, B, C, D, E, F, G, H, I);
tuple_view!(A, B, C, D, E, F, G, H, I, J);
tuple_view!(A, B, C, D, E, F, G, H, I, J, K);
tuple_view!(A, B, C, D, E, F, G, H, I, J, K, L);
