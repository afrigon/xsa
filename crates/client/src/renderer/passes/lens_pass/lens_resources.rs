use render_graph::{BufferHandle, ImageHandle};

#[derive(Clone, Copy)]
pub(in crate::renderer) struct LensResources {
    pub lens: ImageHandle,
    pub visibility: BufferHandle,
    pub star_count: u32,
}
