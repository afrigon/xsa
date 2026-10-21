use std::fmt;

const CUBE_FACE_COUNT: usize = 6;

/// How a texture's layers are arranged: what each layer of a `TextureImage` is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureKind {
    /// A single 2D image.
    Plane,
    /// Six square faces, in Vulkan's face order.
    CubeMap,
    /// Layers of the same size, each its own 2D image.
    Array,
}

impl TextureKind {
    /// Whether a texture of this kind can hold this many layers.
    pub fn holds(self, layer_count: usize) -> bool {
        match self {
            TextureKind::Plane => layer_count == 1,
            TextureKind::CubeMap => layer_count == CUBE_FACE_COUNT,
            TextureKind::Array => layer_count >= 1,
        }
    }
}

impl fmt::Display for TextureKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TextureKind::Plane => "2D texture",
            TextureKind::CubeMap => "cube map",
            TextureKind::Array => "texture array",
        };

        formatter.write_str(name)
    }
}
