use render_graph_vulkan::{ImportedBufferHandle, Subresource};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct CaptureInputs {
    pub output: Subresource,
    pub capture: ImportedBufferHandle,
}
