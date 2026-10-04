#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaterialHandle {
    pub(in crate::renderer) index: usize,
}

impl MaterialHandle {
    pub fn index(self) -> usize {
        self.index
    }
}
