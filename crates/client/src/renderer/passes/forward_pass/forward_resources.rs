use render_graph::ImageHandle;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct ForwardResources {
    pub hdr: ImageHandle,
    pub motion: ImageHandle,
    pub depth: ImageHandle,
}
