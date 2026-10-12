use anyhow::ensure;
use ash::vk;
use texture_image::{TextureFormat, TextureImage};

use crate::vulkan::{Allocator, Buffer, Device, Image, ImageDescription, MemoryLocation};

const CUBE_FACE_COUNT: usize = 6;

pub(in crate::renderer) struct TextureUpload<'a> {
    image: &'a TextureImage,
}

impl<'a> TextureUpload<'a> {
    pub fn new(image: &'a TextureImage) -> Self {
        Self { image }
    }

    pub fn cube_map(&self, device: &Device, allocator: &mut Allocator, name: &str) -> anyhow::Result<Image> {
        ensure!(
            self.image.layers().len() == CUBE_FACE_COUNT,
            "{name}: a cube map needs {CUBE_FACE_COUNT} faces"
        );
        ensure!(
            self.image.width() == self.image.height(),
            "{name}: cube map faces must be square"
        );

        self.upload(device, allocator, name, true)
    }

    pub fn texture(&self, device: &Device, allocator: &mut Allocator, name: &str) -> anyhow::Result<Image> {
        ensure!(
            self.image.layers().len() == 1,
            "{name}: expected a 2D texture, found a cube map"
        );

        self.upload(device, allocator, name, false)
    }

    fn upload(&self, device: &Device, allocator: &mut Allocator, name: &str, cube: bool) -> anyhow::Result<Image> {
        let mut image = Image::new(
            device,
            allocator,
            &ImageDescription {
                name,
                extent: vk::Extent2D {
                    width: self.image.width(),
                    height: self.image.height(),
                },
                format: TextureUpload::vulkan_format(self.image.format()),
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: self.image.mip_levels(),
                cube,
            },
        )?;

        for (layer, data) in self.image.layers().iter().enumerate() {
            if let Err(err) = self.upload_layer(device, allocator, &image, layer as u32, data) {
                unsafe { image.destroy(device, allocator) };
                return Err(err.context(format!("{name}: uploading layer {layer}")));
            }
        }

        Ok(image)
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

    fn upload_layer(
        &self,
        device: &Device,
        allocator: &mut Allocator,
        image: &Image,
        layer: u32,
        data: &[u8],
    ) -> anyhow::Result<()> {
        let mut staging = Buffer::new(
            device,
            allocator,
            "texture staging",
            data.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;
        let result = staging
            .write(data)
            .and_then(|()| self.copy_to_layer(device, &staging, image, layer));
        unsafe { staging.destroy(device, allocator) };

        result
    }

    fn copy_to_layer(&self, device: &Device, staging: &Buffer, image: &Image, layer: u32) -> anyhow::Result<()> {
        let handle = device.handle();
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(device.queue_family());
        let pool = unsafe { handle.create_command_pool(&pool_info, None) }?;
        let result = (|| {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            let command_buffer = unsafe { handle.allocate_command_buffers(&allocate_info) }?[0];
            let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(self.image.mip_levels())
                .base_array_layer(layer)
                .layer_count(1);

            let mut offset = 0;
            let regions: Vec<vk::BufferImageCopy> = (0..self.image.mip_levels())
                .map(|level| {
                    let width = (self.image.width() >> level).max(1);
                    let height = (self.image.height() >> level).max(1);
                    let region = vk::BufferImageCopy::default()
                        .buffer_offset(offset as u64)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(level)
                                .base_array_layer(layer)
                                .layer_count(1),
                        )
                        .image_extent(vk::Extent3D {
                            width,
                            height,
                            depth: 1,
                        });
                    offset += self.image.format().level_size(width, height);

                    region
                })
                .collect();

            unsafe {
                handle.begin_command_buffer(command_buffer, &begin_info)?;
                let to_transfer = [vk::ImageMemoryBarrier2::default()
                    .dst_stage_mask(vk::PipelineStageFlags2::COPY)
                    .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(image.handle())
                    .subresource_range(range)];
                handle.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
                );
                handle.cmd_copy_buffer_to_image(
                    command_buffer,
                    staging.handle(),
                    image.handle(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                );
                let to_shader = [vk::ImageMemoryBarrier2::default()
                    .src_stage_mask(vk::PipelineStageFlags2::COPY)
                    .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
                    .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image(image.handle())
                    .subresource_range(range)];
                handle.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(&to_shader),
                );
                handle.end_command_buffer(command_buffer)?;

                let command_buffers = [vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
                let submit = vk::SubmitInfo2::default().command_buffer_infos(&command_buffers);
                handle.queue_submit2(device.queue(), &[submit], vk::Fence::null())?;
                handle.queue_wait_idle(device.queue())?;
            }

            Ok(())
        })();
        unsafe { handle.destroy_command_pool(pool, None) };

        result
    }
}
