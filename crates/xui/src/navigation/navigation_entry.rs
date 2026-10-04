use super::AnyViewController;

// A controller in the stack, with the id that keeps its screen's identity while it stays there.
pub(crate) struct NavigationEntry {
    pub id: u64,
    pub controller: Box<dyn AnyViewController>,
}
