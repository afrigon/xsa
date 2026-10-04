use crate::layout::StackLayout;
use crate::{DrawList, Environment, Identifiable, Never, Rect, Size, SizeProposal, Subview, View, ViewContext};

// One view per item, like SwiftUI's `ForEach`. Each view keeps its item's identity for when views persist
// between frames.
pub struct ForEach<Id, Content: View> {
    ids: Vec<Id>,
    views: Vec<Content>,
}

impl<Id, Content: View> ForEach<Id, Content> {
    pub fn new<Item: Identifiable<Id = Id>>(
        items: impl IntoIterator<Item = Item>,
        content: impl Fn(Item) -> Content,
    ) -> ForEach<Id, Content> {
        ForEach::with_id(items, Identifiable::id, content)
    }

    pub fn with_id<Item>(
        items: impl IntoIterator<Item = Item>,
        id: impl Fn(&Item) -> Id,
        content: impl Fn(Item) -> Content,
    ) -> ForEach<Id, Content> {
        let mut ids = Vec::new();
        let mut views = Vec::new();

        for item in items {
            ids.push(id(&item));
            views.push(content(item));
        }

        ForEach { ids, views }
    }

    pub fn ids(&self) -> &[Id] {
        &self.ids
    }
}

impl<Id, Content: View> View for ForEach<Id, Content> {
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
        for view in &self.views {
            view.collect_subviews(subviews);
        }
    }
}
