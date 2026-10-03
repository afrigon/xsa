use ash::vk;

use crate::renderer::gpu_context::GpuContext;
use crate::vulkan::{Image, ImageDescription};

const BLOOM_FORMAT: vk::Format = vk::Format::B10G11R11_UFLOAT_PACK32;

pub(super) struct BloomResources {
    pub image: Image,
    pub level_views: Vec<vk::ImageView>,
    pub level_extents: Vec<vk::Extent2D>,
}

impl BloomResources {
    pub fn new(gpu: &mut GpuContext, extent: vk::Extent2D, levels: u32) -> anyhow::Result<Self> {
        let level_extents: Vec<vk::Extent2D> = (1..=levels)
            .map(|level| vk::Extent2D {
                width: (extent.width >> level).max(1),
                height: (extent.height >> level).max(1),
            })
            .collect();
        let mut image = Image::new(
            &gpu.device,
            &mut gpu.allocator,
            &ImageDescription {
                name: "bloom",
                extent: level_extents[0],
                format: BLOOM_FORMAT,
                usage: vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: levels,
                cube: false,
            },
        )?;
        let mut level_views = Vec::with_capacity(levels as usize);

        for level in 0..levels {
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

            match unsafe { gpu.device.handle().create_image_view(&view_info, None) } {
                Ok(view) => level_views.push(view),
                Err(err) => {
                    unsafe {
                        for view in &level_views {
                            gpu.device.handle().destroy_image_view(*view, None);
                        }

                        image.destroy(&gpu.device, &mut gpu.allocator);
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

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            for view in &self.level_views {
                gpu.device.handle().destroy_image_view(*view, None);
            }

            self.image.destroy(&gpu.device, &mut gpu.allocator);
        }
    }
}
