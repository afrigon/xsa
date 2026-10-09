use render_graph::{BufferHandle, ImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct HistogramInputs {
    pub scene_color: ImageHandle,
    pub histogram: BufferHandle,
}
