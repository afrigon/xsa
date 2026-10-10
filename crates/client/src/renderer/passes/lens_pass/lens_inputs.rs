use render_graph::BufferHandle;

#[derive(Clone, Copy)]
pub(in crate::renderer) struct LensInputs {
    pub visibility: BufferHandle,
    pub star_count: u32,
}
