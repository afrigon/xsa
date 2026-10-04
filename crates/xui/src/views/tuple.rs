use crate::layout::StackLayout;
use crate::{Context, Never, Node, SubviewEntry, UpdateContext, View};

// A tuple of views is a list of them, like SwiftUI's `TupleView`: a stack lays out each member's subviews.
macro_rules! tuple_view {
    ($($member:ident),+) => {
        impl<$($member: View),+> View for ($($member,)+) {
            fn body(&self, _context: &Context) -> impl View {
                Never::primitive_body()
            }

            fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
                let mut subviews = Vec::new();
                self.collect_subviews(&mut subviews);
                node.set_layout(StackLayout::list());
                node.update_children(&subviews, context)
            }

            #[allow(non_snake_case)]
            fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
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
