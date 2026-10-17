use ash::vk;
use memory_vulkan::{Buffer, Image, ImageDescription, MemoryContext};
use xui::ATLAS_SIZE;

const FORMAT: vk::Format = vk::Format::R8_UNORM;

pub(crate) struct AtlasImage {
    image: Image,
    initialized: bool,
}

impl AtlasImage {
    pub fn new(memory: &MemoryContext) -> anyhow::Result<AtlasImage> {
        let image = Image::new(
            memory,
            &ImageDescription {
                name: "interface glyph atlas",
                extent: vk::Extent2D {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
                },
                format: FORMAT,
                usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
                aspect: vk::ImageAspectFlags::COLOR,
                mip_levels: 1,
                cube: false,
            },
        )?;

        Ok(AtlasImage {
            image,
            initialized: false,
        })
    }

    pub fn view(&self) -> vk::ImageView {
        self.image.view()
    }

    // Copies `regions` of `staging` into the atlas and leaves it ready for sampling. The first call also clears
    // the atlas, so the image has defined contents and layout before anything samples it.
    pub unsafe fn record_updates(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        staging: &Buffer,
        regions: &[vk::BufferImageCopy],
    ) {
        if self.initialized && regions.is_empty() {
            return;
        }

        if self.initialized {
            self.barrier(
                device,
                command_buffer,
                vk::ImageMemoryBarrier2::default()
                    .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
                    .src_access_mask(vk::AccessFlags2::NONE)
                    .dst_stage_mask(vk::PipelineStageFlags2::COPY)
                    .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE),
            );
        } else {
            self.barrier(
                device,
                command_buffer,
                vk::ImageMemoryBarrier2::default()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_stage_mask(vk::PipelineStageFlags2::NONE)
                    .src_access_mask(vk::AccessFlags2::NONE)
                    .dst_stage_mask(vk::PipelineStageFlags2::CLEAR)
                    .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE),
            );
            unsafe {
                device.cmd_clear_color_image(
                    command_buffer,
                    self.image.handle(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &vk::ClearColorValue::default(),
                    &[AtlasImage::subresource_range()],
                );
            }
            self.barrier(
                device,
                command_buffer,
                vk::ImageMemoryBarrier2::default()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_stage_mask(vk::PipelineStageFlags2::CLEAR)
                    .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                    .dst_stage_mask(vk::PipelineStageFlags2::COPY)
                    .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE),
            );
            self.initialized = true;
        }

        if !regions.is_empty() {
            unsafe {
                device.cmd_copy_buffer_to_image(
                    command_buffer,
                    staging.handle(),
                    self.image.handle(),
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    regions,
                );
            }
        }

        self.barrier(
            device,
            command_buffer,
            vk::ImageMemoryBarrier2::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_stage_mask(vk::PipelineStageFlags2::COPY)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ),
        );
    }

    fn barrier(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, barrier: vk::ImageMemoryBarrier2) {
        let barriers = [barrier
            .image(self.image.handle())
            .subresource_range(AtlasImage::subresource_range())];
        let dependency = vk::DependencyInfo::default().image_memory_barriers(&barriers);
        unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
    }

    fn subresource_range() -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1)
    }
}
