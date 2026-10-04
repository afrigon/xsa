#[derive(Clone, Copy)]
pub struct TextureHandle {
    index: u32,
}

impl TextureHandle {
    pub(in crate::renderer) fn new(index: u32) -> Self {
        Self { index }
    }

    pub fn index(self) -> u32 {
        self.index
    }
}
