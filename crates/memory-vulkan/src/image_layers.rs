use ash::vk;

const CUBE_FACES: u32 = 6;

/// The layers of an image and how its views see them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLayers {
    Single,
    /// Six layers, viewed as a cube.
    Cube,
    /// This many layers, viewed as an array.
    Array {
        count: u32,
    },
}

impl ImageLayers {
    pub fn count(self) -> u32 {
        match self {
            ImageLayers::Single => 1,
            ImageLayers::Cube => CUBE_FACES,
            ImageLayers::Array { count } => count,
        }
    }

    pub(crate) fn create_flags(self) -> vk::ImageCreateFlags {
        match self {
            ImageLayers::Single | ImageLayers::Array { .. } => vk::ImageCreateFlags::empty(),
            ImageLayers::Cube => vk::ImageCreateFlags::CUBE_COMPATIBLE,
        }
    }

    pub(crate) fn view_type(self) -> vk::ImageViewType {
        match self {
            ImageLayers::Single => vk::ImageViewType::TYPE_2D,
            ImageLayers::Cube => vk::ImageViewType::CUBE,
            ImageLayers::Array { .. } => vk::ImageViewType::TYPE_2D_ARRAY,
        }
    }
}
