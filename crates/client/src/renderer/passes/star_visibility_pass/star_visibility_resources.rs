use render_graph::{BufferHandle, ImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct StarVisibilityResources {
    pub depth: ImageHandle,
    pub visibility: BufferHandle,
    pub star_count: u32,
}
