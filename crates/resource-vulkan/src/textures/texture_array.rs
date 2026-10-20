use std::path::Path;

use memory_vulkan::{BindlessTable, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureImage, TextureMetadata};

use crate::textures::{PendingTexture, Texture, TextureUpload};
use crate::{LoadContext, MemoryUsage, PendingResource, ResourceKind};

/// A texture array in GPU memory, sampled through its slot in the texture array binding, with the key/value metadata
/// its file carries.
pub struct TextureArray {
    texture: Texture,
    metadata: TextureMetadata,
}

impl TextureArray {
    /// Where shaders find it in the texture array binding's array.
    pub fn index(&self) -> u32 {
        self.texture.index()
    }

    pub fn metadata(&self) -> &TextureMetadata {
        &self.metadata
    }
}

impl ResourceKind for TextureArray {
    type Options = ColorSpace;

    fn load(
        context: &LoadContext,
        path: &Path,
        color_space: &ColorSpace,
    ) -> anyhow::Result<Box<dyn PendingResource<TextureArray>>> {
        let image = TextureImage::read(path, *color_space)?;
        let metadata = image.all_metadata().clone();

        Ok(Box::new(PendingTextureArray {
            texture: TextureUpload::new(image).array(context)?,
            metadata: Some(metadata),
        }))
    }

    fn memory(&self) -> MemoryUsage {
        self.texture.memory()
    }
}

struct PendingTextureArray {
    texture: PendingTexture,
    // Taken by the array once its upload completes.
    metadata: Option<TextureMetadata>,
}

impl PendingResource<TextureArray> for PendingTextureArray {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<TextureArray>> {
        let Some(texture) = self.texture.poll(bindless)? else {
            return Ok(None);
        };

        Ok(Some(TextureArray {
            texture,
            metadata: self.metadata.take().unwrap_or_default(),
        }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
