use crate::Subresource;

/// An image the graph allocates for the frame being built: created by a pass, or one of a history's pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageHandle {
    pub(crate) index: usize,
}

impl ImageHandle {
    pub fn level(self, level: u32) -> Subresource {
        Subresource {
            image: self.index,
            level: Some(level),
        }
    }
}
