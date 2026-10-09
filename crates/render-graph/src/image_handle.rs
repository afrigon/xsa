use crate::Subresource;

/// An image in the frame being built: created by a pass, a history image or imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageHandle {
    pub(crate) index: usize,
}

impl ImageHandle {
    pub fn level(self, level: u32) -> Subresource {
        Subresource {
            image: self,
            level: Some(level),
        }
    }
}
