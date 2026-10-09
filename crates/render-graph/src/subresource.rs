use crate::{ImageHandle, ImportedImageHandle};

/// An image, or one mip level of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subresource {
    pub(crate) image: usize,
    pub(crate) level: Option<u32>,
}

impl From<ImageHandle> for Subresource {
    fn from(image: ImageHandle) -> Subresource {
        Subresource {
            image: image.index,
            level: None,
        }
    }
}

impl From<ImportedImageHandle> for Subresource {
    fn from(image: ImportedImageHandle) -> Subresource {
        Subresource {
            image: image.index,
            level: None,
        }
    }
}
