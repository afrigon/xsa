use render_graph_vulkan::ImageHandle;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct BloomResources {
    pub scene_color: ImageHandle,
    pub bloom: ImageHandle,
}
