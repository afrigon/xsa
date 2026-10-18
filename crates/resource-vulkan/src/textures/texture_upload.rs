use anyhow::ensure;
use ash::vk;
use memory_vulkan::{BindlessBinding, Image, ImageCopy, ImageDescription, ImageUpload, TexelBlock};
use resource_loader::texture::TextureImage;

use crate::LoadContext;
use crate::textures::PendingTexture;

const CUBE_FACE_COUNT: usize = 6;

// Creates a texture's image, takes its bindless slot and queues the upload of every level of every layer.
pub(crate) struct TextureUpload {
    image: TextureImage,
}

impl TextureUpload {
    pub fn new(image: TextureImage) -> TextureUpload {
        TextureUpload { image }
    }

    pub fn cube_map(self, context: &LoadContext) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.layers().len() == CUBE_FACE_COUNT,
            "a cube map needs {CUBE_FACE_COUNT} faces"
        );
        ensure!(
            self.image.width() == self.image.height(),
            "cube map faces must be square"
        );

        self.upload(context, BindlessBinding::CubeMap)
    }

    pub fn texture(self, context: &LoadContext) -> anyhow::Result<PendingTexture> {
        ensure!(
            self.image.layers().len() == 1,
            "expected a 2D texture, found a cube map"
        );

        self.upload(context, BindlessBinding::Texture)
    }

    fn upload(self, context: &LoadContext, binding: BindlessBinding) -> anyhow::Result<PendingTexture> {
        let format = self.image.format();
        let image = Image::new(
            context.memory,
            &ImageDescription {
                name: context.name,
                extent: vk::Extent2D {
                    width: self.image.width(),
                    height: self.image.height(),
                },
                format: vk::Format::from_raw(format.vulkan_format().value as i32),
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: self.image.mip_levels(),
                cube: binding == BindlessBinding::CubeMap,
                sharing: context.transfer.sharing(),
            },
        )?;
        let slot = context.bindless.allocate(binding)?;
        let copies = self.copies();
        let upload = ImageUpload {
            data: self.image.into_layers(),
            copies,
            texel_block: TexelBlock {
                extent: vk::Extent2D {
                    width: format.block_extent(),
                    height: format.block_extent(),
                },
                size: format.block_size() as u32,
            },
        };
        let ticket = context.transfer.upload_image(image, upload, context.priority);

        Ok(PendingTexture::new(ticket.into(), slot))
    }

    // Each layer holds its mip chain, largest level first, packed without padding.
    fn copies(&self) -> Vec<ImageCopy> {
        let format = self.image.format();
        let mut copies = Vec::new();

        for layer in 0..self.image.layers().len() {
            let mut offset = 0;

            for level in 0..self.image.mip_levels() {
                let width = (self.image.width() >> level).max(1);
                let height = (self.image.height() >> level).max(1);
                copies.push(ImageCopy {
                    data: layer,
                    offset,
                    mip_level: level,
                    array_layer: layer as u32,
                    image_offset: vk::Offset2D::default(),
                    extent: vk::Extent2D { width, height },
                });
                offset += format.level_size(width, height);
            }
        }

        copies
    }
}
