use render_graph::{ImportedBufferHandle, ImportedImageHandle};

#[derive(Clone, Copy)]
pub(super) struct FrameInputs {
    pub output: ImportedImageHandle,
    pub histogram: ImportedBufferHandle,
    pub albedo_histogram: ImportedBufferHandle,
    pub capture: Option<ImportedBufferHandle>,
    pub projected_light_count: u32,
    pub lens_loaded: bool,
}
