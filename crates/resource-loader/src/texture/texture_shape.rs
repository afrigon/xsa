use std::fmt;

const CUBE_FACE_COUNT: usize = 6;

/// How a texture's layers are arranged: what each layer of a `TextureImage` is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureShape {
    /// A single 2D image.
    Plane,
    /// Six square faces, in Vulkan's face order.
    CubeMap,
    /// Layers of the same size, each its own 2D image.
    Array,
}

impl TextureShape {
    /// Whether a texture of this shape can hold this many layers.
    pub fn holds(self, layer_count: usize) -> bool {
        match self {
            TextureShape::Plane => layer_count == 1,
            TextureShape::CubeMap => layer_count == CUBE_FACE_COUNT,
            TextureShape::Array => layer_count >= 1,
        }
    }
}

impl fmt::Display for TextureShape {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TextureShape::Plane => "2D texture",
            TextureShape::CubeMap => "cube map",
            TextureShape::Array => "texture array",
        };

        formatter.write_str(name)
    }
}
