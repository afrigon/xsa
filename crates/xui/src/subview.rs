use std::any::TypeId;

use crate::{Node, SubviewEntry, UpdateContext, View};

// `View` in a form that can sit behind a reference to `dyn`, so containers can hold children of different types.
// Every view implements it.
pub trait Subview {
    fn view_type(&self) -> TypeId;

    fn update_node(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()>;

    fn collect_into<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>);
}

impl<Content: View> Subview for Content {
    fn view_type(&self) -> TypeId {
        TypeId::of::<Content>()
    }

    fn update_node(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.begin_update();
        self.update(node, context)?;
        node.end_update();

        Ok(())
    }

    fn collect_into<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        self.collect_subviews(subviews);
    }
}
