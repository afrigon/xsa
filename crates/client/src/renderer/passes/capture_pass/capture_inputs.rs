use render_graph::{BufferHandle, ImportedImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct CaptureInputs {
    pub output: ImportedImageHandle,
    pub capture: BufferHandle,
}
