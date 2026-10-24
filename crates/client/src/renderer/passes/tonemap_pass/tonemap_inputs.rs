use render_graph_vulkan::{ImageHandle, Subresource};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct TonemapInputs {
    pub scene_color: ImageHandle,
    pub bloom: Option<ImageHandle>,
    pub lens: Option<ImageHandle>,
    pub output: Subresource,
}
