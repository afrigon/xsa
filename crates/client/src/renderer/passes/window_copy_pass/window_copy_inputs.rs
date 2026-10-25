use render_graph_vulkan::{ImageHandle, Subresource};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct WindowCopyInputs {
    pub source: ImageHandle,
    pub window: Subresource,
}
