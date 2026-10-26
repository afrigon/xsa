use std::ops::Range;
use std::path::Path;

use memory_vulkan::{BindlessSlot, Image};
use resource_loader::texture::{ColorSpace, TextureImage};

use crate::textures::{PendingTexture, TextureUpload};
use crate::{LoadContext, MemoryUsage, PendingResource, ResourceKind};

/// A 2D texture in GPU memory, sampled through its bindless slot.
pub struct Texture {
    // Declaration order is destruction order: the slot returns once frames reading the image finish.
    slot: BindlessSlot,
    image: Image,
}

impl Texture {
    /// Creates `image`'s texture and queues its upload, for kinds that read the file themselves.
    pub fn upload(context: &LoadContext, image: TextureImage) -> anyhow::Result<PendingTexture> {
        TextureUpload::new(image).texture(context)
    }

    /// Creates an image of a cube map's `levels`, its faces as the six layers of an array in the texture array binding,
    /// and queues the upload of the levels `image` holds, the first of `levels`. For `StreamedMips` kinds that read
    /// texels by face rather than sample by direction.
    pub fn upload_faces(
        context: &LoadContext,
        image: TextureImage,
        levels: Range<u32>,
    ) -> anyhow::Result<PendingTexture> {
        TextureUpload::new(image).faces(context, levels)
    }

    pub(crate) fn new(image: Image, slot: BindlessSlot) -> Texture {
        Texture { slot, image }
    }

    /// Where shaders find it in its binding's array.
    pub fn index(&self) -> u32 {
        self.slot.index()
    }

    pub fn image(&self) -> &Image {
        &self.image
    }
}

impl ResourceKind for Texture {
    type Options = ColorSpace;

    fn load(
        context: &LoadContext,
        path: &Path,
        color_space: &ColorSpace,
    ) -> anyhow::Result<Box<dyn PendingResource<Texture>>> {
        let image = TextureImage::read(path, *color_space)?;

        Ok(Box::new(Texture::upload(context, image)?))
    }

    fn memory(&self) -> MemoryUsage {
        MemoryUsage {
            video: self.image.size(),
            ram: 0,
        }
    }
}
