#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaterialHandle {
    pub(in crate::renderer) index: usize,
}
