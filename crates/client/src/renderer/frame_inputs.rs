use render_graph::{ImportedBufferHandle, ImportedImageHandle};

#[derive(Clone, Copy)]
pub(super) struct FrameInputs {
    pub output: ImportedImageHandle,
    pub histogram: ImportedBufferHandle,
    pub capture: Option<ImportedBufferHandle>,
}
