use ash::vk;
use xui::ATLAS_SIZE;

use crate::host_buffer::HostBuffer;
use crate::memory_types::MemoryTypes;

const FORMAT: vk::Format = vk::Format::R8_UNORM;

pub(crate) struct AtlasImage {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    initialized: bool,
}

impl AtlasImage {
    pub unsafe fn new(device: &ash::Device, memory_types: &MemoryTypes) -> anyhow::Result<AtlasImage> {
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(FORMAT)
            .extent(vk::Extent3D {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = unsafe { device.create_image(&info, None) }?;
        let requirements = unsafe { device.get_image_memory_requirements(image) };
        let memory = match unsafe { memory_types.allocate(device, requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL) }
        {
            Ok(memory) => memory,
            Err(error) => {
                unsafe { device.destroy_image(image, None) };
                return Err(error);
            }
        };
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(FORMAT)
            .subresource_range(AtlasImage::subresource_range());
        let view = unsafe {
            device
                .bind_image_memory(image, memory, 0)
                .and_then(|()| device.create_image_view(&view_info, None))
        };
        let view = match view {
            Ok(view) => view,
            Err(error) => {
                unsafe {
                    device.destroy_image(image, None);
                    device.free_memory(memory, None);
                }
                return Err(error.into());
            }
        };

        Ok(AtlasImage {
            image,
            memory,
            view,
            initialized: false,
        })
    }

    pub fn view(&self) -> vk::ImageView {
        self.view
    }

    // Copies `regions` of `staging` into the atlas and leaves it ready for sampling. The first call also clears
    // the atlas, so the image has defined contents and layout before anything samples it.
    pub unsafe fn record_updates(
        &mut self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        staging: &HostBuffer,
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
                    self.image,
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
                    self.image,
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

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }

    fn barrier(&self, device: &ash::Device, command_buffer: vk::CommandBuffer, barrier: vk::ImageMemoryBarrier2) {
        let barriers = [barrier
            .image(self.image)
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
