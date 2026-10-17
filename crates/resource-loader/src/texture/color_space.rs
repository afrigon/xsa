/// How a texture's values are meant: colors are stored sRGB-encoded, data (normals, heights, parameters) linearly.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ColorSpace {
    Srgb,
    Linear,
}
