use anyhow::ensure;
use ash::vk;
use resource_loader::texture::{TextureFormat, TextureImage};

use memory_vulkan::{Image, ImageCopy, ImageDescription, ImageUpload, TexelBlock, TransferPriority, UploadTicket};

use crate::renderer::gpu_context::GpuContext;

const CUBE_FACE_COUNT: usize = 6;

// Creates a texture's image and queues the upload of every level of every layer.
pub(in crate::renderer) struct TextureUpload {
    image: TextureImage,
}

impl TextureUpload {
    pub fn new(image: TextureImage) -> Self {
        Self { image }
    }

    pub fn cube_map(self, gpu: &GpuContext, name: &str) -> anyhow::Result<UploadTicket<Image>> {
        ensure!(
            self.image.layers().len() == CUBE_FACE_COUNT,
            "{name}: a cube map needs {CUBE_FACE_COUNT} faces"
        );
        ensure!(
            self.image.width() == self.image.height(),
            "{name}: cube map faces must be square"
        );

        self.upload(gpu, name, true)
    }

    pub fn texture(self, gpu: &GpuContext, name: &str) -> anyhow::Result<UploadTicket<Image>> {
        ensure!(
            self.image.layers().len() == 1,
            "{name}: expected a 2D texture, found a cube map"
        );

        self.upload(gpu, name, false)
    }

    fn upload(self, gpu: &GpuContext, name: &str, cube: bool) -> anyhow::Result<UploadTicket<Image>> {
        let format = self.image.format();
        let image = Image::new(
            &gpu.memory,
            &ImageDescription {
                name,
                extent: vk::Extent2D {
                    width: self.image.width(),
                    height: self.image.height(),
                },
                format: TextureUpload::vulkan_format(format),
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: self.image.mip_levels(),
                cube,
                sharing: gpu.transfer.sharing(),
            },
        )?;
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

        Ok(gpu.transfer.upload_image(image, upload, TransferPriority::High))
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

    fn vulkan_format(format: TextureFormat) -> vk::Format {
        match format {
            TextureFormat::Bc4Unorm => vk::Format::BC4_UNORM_BLOCK,
            TextureFormat::Bc5Unorm => vk::Format::BC5_UNORM_BLOCK,
            TextureFormat::Bc7Unorm => vk::Format::BC7_UNORM_BLOCK,
            TextureFormat::Bc7Srgb => vk::Format::BC7_SRGB_BLOCK,
            TextureFormat::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
            TextureFormat::Rgba8Srgb => vk::Format::R8G8B8A8_SRGB,
            TextureFormat::Rgba16Float => vk::Format::R16G16B16A16_SFLOAT,
            TextureFormat::R16Unorm => vk::Format::R16_UNORM,
            TextureFormat::Rg16Unorm => vk::Format::R16G16_UNORM,
        }
    }
}
