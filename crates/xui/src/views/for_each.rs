use std::hash::{DefaultHasher, Hash, Hasher};

use crate::layout::StackLayout;
use crate::{Context, Identifiable, IdentifiedView, Never, Node, SubviewEntry, UpdateContext, View};

// One view per item, like SwiftUI's `ForEach`. Each view is identified by its item's id, so it keeps its node
// when items are inserted, removed or reordered.
pub struct ForEach<Id: Hash + 'static, Content: View> {
    ids: Vec<Id>,
    views: Vec<Content>,
}

impl<Id: Hash + 'static, Content: View> ForEach<Id, Content> {
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

impl<Id: Hash + 'static, Content: View> View for ForEach<Id, Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let mut subviews = Vec::new();
        self.collect_subviews(&mut subviews);
        node.set_layout(StackLayout::list());
        node.update_children(&subviews, context)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        for (id, view) in self.ids.iter().zip(&self.views) {
            let mut hasher = DefaultHasher::new();
            id.hash(&mut hasher);
            let first = subviews.len();
            view.collect_subviews(subviews);
            IdentifiedView::<Content>::key_subviews(subviews, first, hasher.finish());
        }
    }
}
