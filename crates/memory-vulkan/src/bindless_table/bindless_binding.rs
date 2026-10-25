use ash::vk;

/// An array of the bindless table, by what its slots hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindlessBinding {
    Texture,
    CubeMap,
    StorageImage,
    TextureArray,
    Volume,
}

impl BindlessBinding {
    pub(crate) const ALL: [BindlessBinding; 5] = [
        BindlessBinding::Texture,
        BindlessBinding::CubeMap,
        BindlessBinding::StorageImage,
        BindlessBinding::TextureArray,
        BindlessBinding::Volume,
    ];

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn binding(self) -> u32 {
        match self {
            BindlessBinding::CubeMap => 1,
            BindlessBinding::Texture => 3,
            BindlessBinding::StorageImage => 4,
            BindlessBinding::TextureArray => 5,
            BindlessBinding::Volume => 7,
        }
    }

    pub(crate) fn capacity(self) -> u32 {
        match self {
            BindlessBinding::Texture => 2048,
            BindlessBinding::CubeMap => 64,
            BindlessBinding::StorageImage => 64,
            BindlessBinding::TextureArray => 256,
            BindlessBinding::Volume => 16,
        }
    }

    pub(crate) fn descriptor_type(self) -> vk::DescriptorType {
        match self {
            BindlessBinding::Texture
            | BindlessBinding::CubeMap
            | BindlessBinding::TextureArray
            | BindlessBinding::Volume => vk::DescriptorType::SAMPLED_IMAGE,
            BindlessBinding::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
        }
    }
}
