use crate::Subview;

// One child view to reconcile, with the explicit key that identifies it, if any.
pub struct SubviewEntry<'a> {
    pub view: &'a dyn Subview,
    pub key: Option<u64>,
}

impl<'a> SubviewEntry<'a> {
    pub fn new(view: &'a dyn Subview) -> SubviewEntry<'a> {
        SubviewEntry { view, key: None }
    }
}
