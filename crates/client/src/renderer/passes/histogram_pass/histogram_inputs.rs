use render_graph::{ImageHandle, ImportedBufferHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct HistogramInputs {
    pub scene_color: ImageHandle,
    pub illuminance: ImageHandle,
    pub lens: Option<ImageHandle>,
    pub histogram: ImportedBufferHandle,
    pub albedo_histogram: ImportedBufferHandle,
}
