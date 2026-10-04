use std::any::TypeId;

// What makes a view the same view as last frame: its type, and its explicit key (a ForEach id or `.id()`) when
// it has one. Unkeyed views are matched by their position among their siblings.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct NodeIdentity {
    pub view_type: TypeId,
    pub key: Option<u64>,
}
