use super::{DdsLoader, Ktx2Loader, PngLoader, TextureImage};

/// The texture file extensions read, in order of preference when several files provide the same texture.
pub const TEXTURE_EXTENSIONS: [&str; 3] = [Ktx2Loader::EXTENSION, DdsLoader::EXTENSION, PngLoader::EXTENSION];

/// Reads one texture file format, keeping the texel format the file stores.
pub trait TextureLoader {
    const EXTENSION: &'static str;

    fn load(bytes: &[u8]) -> anyhow::Result<TextureImage>;
}
