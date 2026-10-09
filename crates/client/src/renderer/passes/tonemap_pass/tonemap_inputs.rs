use render_graph::{ImageHandle, ImportedImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct TonemapInputs {
    pub scene_color: ImageHandle,
    pub bloom: Option<ImageHandle>,
    pub output: ImportedImageHandle,
}
