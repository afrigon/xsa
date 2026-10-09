use crate::ImageHandle;

/// An image, or one mip level of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subresource {
    pub(crate) image: ImageHandle,
    pub(crate) level: Option<u32>,
}

impl From<ImageHandle> for Subresource {
    fn from(image: ImageHandle) -> Subresource {
        Subresource { image, level: None }
    }
}
