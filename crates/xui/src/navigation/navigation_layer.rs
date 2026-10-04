// One view controller's view as the navigation controller draws it this frame.
pub(crate) struct NavigationLayer<Content> {
    pub id: u64,
    pub view: Content,
}
