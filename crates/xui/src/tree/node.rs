use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

use super::{NodeIdentity, ProposalKey};
use crate::state::StateSlots;
use crate::{
    Context, DrawList, Environment, LayoutContext, NodeLayout, PointerEvent, Rect, Size, SizeProposal, SubviewEntry,
    UpdateContext,
};

// The part of a view that outlives a frame. Each frame the update pass matches the new views with last frame's
// nodes; a node whose inputs or children changed is marked changed and forgets its measured sizes, so layout
// recomputes only what changed.
pub struct Node {
    identity: NodeIdentity,
    layout: Option<Box<dyn NodeLayout>>,
    children: Vec<Node>,
    sizes: HashMap<ProposalKey, Size>,
    bounds: Rect,
    states: Rc<StateSlots>,
    changed: bool,
}

impl Node {
    pub(crate) fn new(view_type: TypeId, key: Option<u64>) -> Node {
        Node {
            identity: NodeIdentity { view_type, key },
            layout: None,
            children: Vec::new(),
            sizes: HashMap::new(),
            bounds: Rect::default(),
            states: Rc::default(),
            changed: true,
        }
    }

    pub(crate) fn view_type(&self) -> TypeId {
        self.identity.view_type
    }

    // The context the node's view builds its body with; its state lives as long as the node.
    pub fn context(&self, environment: &Environment) -> Context {
        Context::new(environment.clone(), self.states.clone())
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
        self.bounds = bounds;

        match self.layout.as_deref_mut() {
            Some(layout) => layout.place(bounds, &mut self.children, context, draw_list),
            None => Ok(()),
        }
    }

    // Offers `event` to the nodes from the topmost down, against where they were last placed: later children
    // before earlier ones, children before their parent. Returns whether a node claimed it; nodes below a
    // claiming one still see the event, told it was claimed (so a button under the pointer stops hovering).
    pub(crate) fn handle_pointer(&mut self, event: &PointerEvent, claimed: bool) -> bool {
        let mut claimed = claimed;

        for child in self.children.iter_mut().rev() {
            claimed |= child.handle_pointer(event, claimed);
        }

        if let Some(layout) = self.layout.as_deref_mut() {
            claimed |= layout.handle_pointer(self.bounds, event, claimed);
        }

        claimed
    }
}

// A child from last frame, with the position it had then.
struct PreviousNode {
    index: usize,
    node: Node,
}
