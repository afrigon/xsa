use render_graph::{BufferHandle, ImportedImageHandle};

#[derive(Clone, Copy)]
pub(super) struct FrameInputs {
    pub output: ImportedImageHandle,
    pub histogram: BufferHandle,
    pub capture: Option<BufferHandle>,
}
