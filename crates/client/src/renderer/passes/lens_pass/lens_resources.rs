use render_graph::ImageHandle;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct LensResources {
    pub lens: ImageHandle,
    pub light_count: u32,
}
