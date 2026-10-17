use crate::{ImageCopy, TexelBlock};

/// What to write into an image: owned bytes and the regions they fill.
pub struct ImageUpload {
    pub data: Vec<Vec<u8>>,
    pub copies: Vec<ImageCopy>,
    pub texel_block: TexelBlock,
}
