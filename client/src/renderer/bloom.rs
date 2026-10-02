use ash::vk;

use super::barriers;
use super::gpu_data::{self, BloomPushConstants};
use crate::vulkan::{Allocator, BindlessTextures, ComputePipeline, Device, Image, ImageDescription};

// Must match bloomWeightSum in tonemap.slang.
pub(super) const BLOOM_LEVELS: u32 = 6;
const BLOOM_FORMAT: vk::Format = vk::Format::B10G11R11_UFLOAT_PACK32;
const WORKGROUP_SIZE: u32 = 8;

pub(super) struct BloomPipelines<'a> {
    pub downsample: &'a ComputePipeline,
    pub upsample: &'a ComputePipeline,
}

// Level 0 is half the window's resolution; each further level halves again.
pub(super) struct Bloom {
    resources: BloomResources,
    texture: u32,
    storage_images: Vec<u32>,
}

struct BloomResources {
    image: Image,
    level_views: Vec<vk::ImageView>,
    level_extents: Vec<vk::Extent2D>,
}

struct BloomPass {
    source_texture: u32,
    source_level: u32,
    target_level: usize,
    karis_average: bool,
}

impl Bloom {
    pub fn new(
        device: &Device,
        allocator: &mut Allocator,
        bindless: &mut BindlessTextures,
        extent: vk::Extent2D,
    ) -> anyhow::Result<Self> {
        let resources = BloomResources::new(device, allocator, extent)?;
        let texture = bindless.add_texture(device, resources.image.view(), vk::ImageLayout::GENERAL)?;
        let storage_images = resources
            .level_views
            .iter()
            .map(|view| bindless.add_storage_image(device, *view))
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            resources,
            texture,
            storage_images,
        })
    }

    pub fn recreate(
        &mut self,
        device: &Device,
        allocator: &mut Allocator,
        bindless: &BindlessTextures,
        extent: vk::Extent2D,
    ) -> anyhow::Result<()> {
        let resources = BloomResources::new(device, allocator, extent)?;
        bindless.set_texture(device, self.texture, resources.image.view(), vk::ImageLayout::GENERAL);
        for (slot, view) in self.storage_images.iter().zip(&resources.level_views) {
            bindless.set_storage_image(device, *slot, *view);
        }
        let mut old = std::mem::replace(&mut self.resources, resources);
        unsafe { old.destroy(device, allocator) };
        Ok(())
    }

    pub fn texture(&self) -> u32 {
        self.texture
    }

    pub fn record(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        pipelines: &BloomPipelines,
        hdr_texture: u32,
    ) {
        let image = self.resources.image.handle();
        barriers::bloom_start(device, command_buffer, image);
        for level in 0..BLOOM_LEVELS as usize {
            let first = level == 0;
            let pass = BloomPass {
                source_texture: if first { hdr_texture } else { self.texture },
                source_level: if first { 0 } else { level as u32 - 1 },
                target_level: level,
                karis_average: first,
            };
            self.dispatch(device, command_buffer, pipelines.downsample, &pass);
            barriers::bloom_between_passes(device, command_buffer, image);
        }
        for level in (0..BLOOM_LEVELS as usize - 1).rev() {
            let pass = BloomPass {
                source_texture: self.texture,
                source_level: level as u32 + 1,
                target_level: level,
                karis_average: false,
            };
            self.dispatch(device, command_buffer, pipelines.upsample, &pass);
            barriers::bloom_between_passes(device, command_buffer, image);
        }
        barriers::bloom_to_fragment(device, command_buffer, image);
    }

    fn dispatch(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        pipeline: &ComputePipeline,
        pass: &BloomPass,
    ) {
        let push_constants = BloomPushConstants {
            source_texture: pass.source_texture,
            source_level: pass.source_level,
            target_image: self.storage_images[pass.target_level],
            karis_average: u32::from(pass.karis_average),
        };
        let extent = self.resources.level_extents[pass.target_level];
        unsafe {
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline.handle());
            device.cmd_push_constants(
                command_buffer,
                pipeline.layout(),
                vk::ShaderStageFlags::COMPUTE,
                0,
                gpu_data::as_bytes(&push_constants),
            );
            device.cmd_dispatch(
                command_buffer,
                extent.width.div_ceil(WORKGROUP_SIZE),
                extent.height.div_ceil(WORKGROUP_SIZE),
                1,
            );
        }
    }

    pub unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        unsafe { self.resources.destroy(device, allocator) };
    }
}

impl BloomResources {
    fn new(device: &Device, allocator: &mut Allocator, extent: vk::Extent2D) -> anyhow::Result<Self> {
        let level_extents: Vec<vk::Extent2D> = (1..=BLOOM_LEVELS)
            .map(|level| vk::Extent2D {
                width: (extent.width >> level).max(1),
                height: (extent.height >> level).max(1),
            })
            .collect();
        let mut image = Image::new(
            device,
            allocator,
            &ImageDescription {
                name: "bloom",
                extent: level_extents[0],
                format: BLOOM_FORMAT,
                usage: vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: BLOOM_LEVELS,
                cube: false,
            },
        )?;
        let mut level_views = Vec::with_capacity(BLOOM_LEVELS as usize);
        for level in 0..BLOOM_LEVELS {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image.handle())
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(BLOOM_FORMAT)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(level)
                        .level_count(1)
                        .layer_count(1),
                );
            match unsafe { device.handle().create_image_view(&view_info, None) } {
                Ok(view) => level_views.push(view),
                Err(err) => {
                    unsafe {
                        for view in &level_views {
                            device.handle().destroy_image_view(*view, None);
                        }
                        image.destroy(device, allocator);
                    }
                    return Err(err.into());
                }
            }
        }
        Ok(Self {
            image,
            level_views,
            level_extents,
        })
    }

    unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            for view in &self.level_views {
                device.handle().destroy_image_view(*view, None);
            }
            self.image.destroy(device, allocator);
        }
    }
}
