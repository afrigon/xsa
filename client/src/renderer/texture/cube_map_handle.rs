#[derive(Clone, Copy)]
pub struct CubeMapHandle {
    index: u32,
}

impl CubeMapHandle {
    pub(in crate::renderer) fn new(index: u32) -> Self {
        Self { index }
    }

    pub fn index(self) -> u32 {
        self.index
    }
}
