use render_graph_vulkan::{ImageHandle, ImportedBufferHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct HistogramInputs {
    pub scene_color: ImageHandle,
    pub lens: Option<ImageHandle>,
    pub histogram: ImportedBufferHandle,
}
