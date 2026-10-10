use render_graph::ImageHandle;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct StarVisibilityInputs {
    pub depth: ImageHandle,
    pub star_count: u32,
}
