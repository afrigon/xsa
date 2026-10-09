use render_graph::{BufferHandle, ImageHandle};

#[derive(Clone, Copy)]
pub(super) struct FrameInputs {
    pub output: ImageHandle,
    pub histogram: BufferHandle,
    pub capture: Option<BufferHandle>,
}
