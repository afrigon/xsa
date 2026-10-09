use render_graph::ImageHandle;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct BloomResources {
    pub scene_color: ImageHandle,
    pub bloom: ImageHandle,
}
