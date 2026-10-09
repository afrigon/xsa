/// One of the images the graph allocates, stable across frames and resizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId {
    pub(crate) index: usize,
}

impl ImageId {
    pub fn index(self) -> usize {
        self.index
    }
}
