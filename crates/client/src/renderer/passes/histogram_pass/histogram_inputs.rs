use render_graph::{ImageHandle, ImportedBufferHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct HistogramInputs {
    pub scene_color: ImageHandle,
    pub histogram: ImportedBufferHandle,
}
