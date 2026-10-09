use render_graph::{BufferHandle, ImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct CaptureInputs {
    pub output: ImageHandle,
    pub capture: BufferHandle,
}
