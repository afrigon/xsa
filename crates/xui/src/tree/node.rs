use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::{NodeIdentity, ProposalKey};
use crate::{DrawList, LayoutContext, NodeLayout, Rect, Size, SizeProposal, SubviewEntry, UpdateContext};

// The part of a view that outlives a frame. Each frame the update pass matches the new views with last frame's
// nodes; a node whose inputs or children changed is marked changed and forgets its measured sizes, so layout
// recomputes only what changed.
pub struct Node {
    identity: NodeIdentity,
    layout: Option<Box<dyn NodeLayout>>,
    children: Vec<Node>,
    sizes: HashMap<ProposalKey, Size>,
    changed: bool,
}

impl Node {
    pub(crate) fn new(view_type: TypeId, key: Option<u64>) -> Node {
        Node {
            identity: NodeIdentity { view_type, key },
            layout: None,
            children: Vec::new(),
            sizes: HashMap::new(),
            changed: true,
        }
    }

    pub(crate) fn view_type(&self) -> TypeId {
        self.identity.view_type
    }

    // The node's layout, created on first use.
    pub fn layout_mut<Layout: NodeLayout>(&mut self, create: impl FnOnce() -> Layout) -> &mut Layout {
        let reusable = self
            .layout
            .as_deref_mut()
            .is_some_and(|layout| (layout as &mut dyn Any).is::<Layout>());

        if !reusable {
            self.layout = Some(Box::new(create()));
            self.changed = true;
        }

        let layout = self.layout.as_deref_mut().expect("the layout was set above");
        (layout as &mut dyn Any)
            .downcast_mut::<Layout>()
            .expect("the layout has this type")
    }

    // Replaces the node's layout when it differs from the current one, marking the node changed.
    pub fn set_layout<Layout: NodeLayout + PartialEq>(&mut self, layout: Layout) {
        let current = self
            .layout
            .as_deref_mut()
            .and_then(|current| (current as &mut dyn Any).downcast_mut::<Layout>());

        if current.is_some_and(|current| *current == layout) {
            return;
        }

        self.layout = Some(Box::new(layout));
        self.changed = true;
    }

    // Marks an input that changes the node's size; its sizes are measured again.
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub(crate) fn begin_update(&mut self) {
        self.changed = false;
    }

    pub(crate) fn end_update(&mut self) {
        if self.changed {
            self.sizes.clear();
        }
    }

    // Matches `entries` with the previous children: by key when they have one, otherwise by position among the
    // unkeyed children, and in both cases only with a node of the same view type. Unmatched entries get a new
    // node; unmatched previous children are dropped.
    pub fn update_children(&mut self, entries: &[SubviewEntry], context: &mut UpdateContext) -> anyhow::Result<()> {
        let previous = std::mem::take(&mut self.children);
        let mut structure_changed = previous.len() != entries.len();
        let mut keyed = HashMap::new();
        let mut unkeyed = Vec::new();

        for (index, node) in previous.into_iter().enumerate() {
            match node.identity.key {
                Some(_) => {
                    keyed.insert(node.identity, PreviousNode { index, node });
                }
                None => unkeyed.push(PreviousNode { index, node }),
            }
        }

        let mut unkeyed = unkeyed.into_iter();
        let mut children = Vec::with_capacity(entries.len());

        for (index, entry) in entries.iter().enumerate() {
            let identity = NodeIdentity {
                view_type: entry.view.view_type(),
                key: entry.key,
            };
            let previous = match entry.key {
                Some(_) => keyed.remove(&identity),
                None => unkeyed
                    .next()
                    .filter(|previous| previous.node.identity.view_type == identity.view_type),
            };
            let mut node = match previous {
                Some(previous) => {
                    structure_changed |= previous.index != index;
                    previous.node
                }
                None => {
                    structure_changed = true;
                    Node::new(identity.view_type, identity.key)
                }
            };
            entry.view.update_node(&mut node, context)?;
            self.changed |= node.changed;
            children.push(node);
        }

        self.changed |= structure_changed;
        self.children = children;

        Ok(())
    }

    pub fn size_that_fits(&mut self, proposal: SizeProposal, context: &mut LayoutContext) -> anyhow::Result<Size> {
        let key = ProposalKey::from(proposal);

        if let Some(size) = self.sizes.get(&key) {
            return Ok(*size);
        }

        let size = match self.layout.as_deref_mut() {
            Some(layout) => layout.size_that_fits(proposal, &mut self.children, context)?,
            None => Size::default(),
        };
        self.sizes.insert(key, size);

        Ok(size)
    }

    pub fn place(&mut self, bounds: Rect, context: &mut LayoutContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        match self.layout.as_deref_mut() {
            Some(layout) => layout.place(bounds, &mut self.children, context, draw_list),
            None => Ok(()),
        }
    }
}

// A child from last frame, with the position it had then.
struct PreviousNode {
    index: usize,
    node: Node,
}
