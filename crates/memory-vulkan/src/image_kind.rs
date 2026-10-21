use ash::vk;

const CUBE_FACES: u32 = 6;

/// What an image holds beyond one plane of texels, and how its views see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageKind {
    Single,
    /// Six layers, viewed as a cube.
    Cube,
    /// This many layers, viewed as an array.
    Array {
        count: u32,
    },
    /// This many slices deep, viewed as a volume filtered across them.
    Volume {
        depth: u32,
    },
}

impl ImageKind {
    /// Its array layers.
    pub fn count(self) -> u32 {
        match self {
            ImageKind::Single | ImageKind::Volume { .. } => 1,
            ImageKind::Cube => CUBE_FACES,
            ImageKind::Array { count } => count,
        }
    }

    pub(crate) fn depth(self) -> u32 {
        match self {
            ImageKind::Volume { depth } => depth,
            ImageKind::Single | ImageKind::Cube | ImageKind::Array { .. } => 1,
        }
    }

    pub(crate) fn create_flags(self) -> vk::ImageCreateFlags {
        match self {
            ImageKind::Single | ImageKind::Array { .. } | ImageKind::Volume { .. } => vk::ImageCreateFlags::empty(),
            ImageKind::Cube => vk::ImageCreateFlags::CUBE_COMPATIBLE,
        }
    }

    pub(crate) fn image_type(self) -> vk::ImageType {
        match self {
            ImageKind::Volume { .. } => vk::ImageType::TYPE_3D,
            ImageKind::Single | ImageKind::Cube | ImageKind::Array { .. } => vk::ImageType::TYPE_2D,
        }
    }

    pub(crate) fn view_type(self) -> vk::ImageViewType {
        match self {
            ImageKind::Single => vk::ImageViewType::TYPE_2D,
            ImageKind::Cube => vk::ImageViewType::CUBE,
            ImageKind::Array { .. } => vk::ImageViewType::TYPE_2D_ARRAY,
            ImageKind::Volume { .. } => vk::ImageViewType::TYPE_3D,
        }
    }
}
