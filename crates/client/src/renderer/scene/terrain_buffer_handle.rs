#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TerrainBufferHandle {
    pub(in crate::renderer) index: usize,
}
