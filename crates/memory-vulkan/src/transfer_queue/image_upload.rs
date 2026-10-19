use crate::{ImageCopy, TexelBlock};

/// What to write into an image: byte arrays and the regions they fill. Each array is any owner of bytes, such as a
/// `Vec<u8>` or an `Arc<[u8]>` shared with a cache.
pub struct ImageUpload {
    pub data: Vec<Box<dyn AsRef<[u8]> + Send + Sync>>,
    pub copies: Vec<ImageCopy>,
    pub texel_block: TexelBlock,
}
