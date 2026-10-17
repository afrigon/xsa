use render_graph_vulkan::{ImportedBufferHandle, ImportedImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct CaptureInputs {
    pub output: ImportedImageHandle,
    pub capture: ImportedBufferHandle,
}
