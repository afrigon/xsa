use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Context, Never, Node, PassthroughLayout, SubviewEntry, UpdateContext, View};

// A view with an explicit identity: when the id changes it is a different view, with a new node.
pub struct IdentifiedView<Content: View> {
    content: Content,
    id: u64,
}

impl<Content: View> IdentifiedView<Content> {
    pub fn new(content: Content, id: impl Hash) -> IdentifiedView<Content> {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        IdentifiedView {
            content,
            id: hasher.finish(),
        }
    }

    // Keys `subviews` from `first` on with `id`, keeping apart the subviews one item contributes.
    pub(crate) fn key_subviews(subviews: &mut [SubviewEntry], first: usize, id: u64) {
        for (offset, entry) in subviews[first..].iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            id.hash(&mut hasher);
            entry.key.unwrap_or(offset as u64).hash(&mut hasher);
            entry.key = Some(hasher.finish());
        }
    }
}

impl<Content: View> View for IdentifiedView<Content> {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.layout_mut(|| PassthroughLayout);
        node.update_children(
            &[SubviewEntry {
                view: &self.content,
                key: Some(self.id),
            }],
            context,
        )
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>) {
        let first = subviews.len();
        self.content.collect_subviews(subviews);
        IdentifiedView::<Content>::key_subviews(subviews, first, self.id);
    }
}
