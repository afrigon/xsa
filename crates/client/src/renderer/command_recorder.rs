use ash::vk;

use super::gpu_data::GpuData;
use crate::vulkan::{Buffer, ComputePipeline, Device, GraphicsPipeline};

const FULLSCREEN_VERTEX_COUNT: u32 = 3;
const GRAPHICS_STAGES: vk::ShaderStageFlags =
    vk::ShaderStageFlags::from_raw(vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw());
pub(super) struct CommandRecorder<'a> {
    device: &'a Device,
    command_buffer: vk::CommandBuffer,
}

impl<'a> CommandRecorder<'a> {
    pub fn new(device: &'a Device, command_buffer: vk::CommandBuffer) -> CommandRecorder<'a> {
        CommandRecorder { device, command_buffer }
    }

    pub fn bind_graphics(&self, pipeline: &GraphicsPipeline, descriptor_set: vk::DescriptorSet) {
        self.bind(
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.handle(),
            pipeline.layout(),
            descriptor_set,
        );
    }

    pub fn bind_compute(&self, pipeline: &ComputePipeline, descriptor_set: vk::DescriptorSet) {
        self.bind(
            vk::PipelineBindPoint::COMPUTE,
            pipeline.handle(),
            pipeline.layout(),
            descriptor_set,
        );
    }

    pub fn push_graphics_constants(&self, pipeline: &GraphicsPipeline, data: &impl GpuData) {
        self.push_constants(pipeline.layout(), GRAPHICS_STAGES, data);
    }

    pub fn push_compute_constants(&self, pipeline: &ComputePipeline, data: &impl GpuData) {
        self.push_constants(pipeline.layout(), vk::ShaderStageFlags::COMPUTE, data);
    }

    pub fn bind_mesh(&self, vertices: &Buffer, indices: &Buffer) {
        let device = self.device.handle();

        unsafe {
            device.cmd_bind_vertex_buffers(self.command_buffer, 0, &[vertices.handle()], &[0]);
            device.cmd_bind_index_buffer(self.command_buffer, indices.handle(), 0, vk::IndexType::UINT32);
        }
    }

    pub fn draw_indexed(&self, index_count: u32) {
        unsafe {
            self.device
                .handle()
                .cmd_draw_indexed(self.command_buffer, index_count, 1, 0, 0, 0)
        };
    }

    pub fn draw_fullscreen(&self) {
        unsafe {
            self.device
                .handle()
                .cmd_draw(self.command_buffer, FULLSCREEN_VERTEX_COUNT, 1, 0, 0)
        };
    }

    pub fn dispatch(&self, extent: vk::Extent2D, workgroup_size: u32) {
        unsafe {
            self.device.handle().cmd_dispatch(
                self.command_buffer,
                extent.width.div_ceil(workgroup_size),
                extent.height.div_ceil(workgroup_size),
                1,
            )
        };
    }

    pub fn clear_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.device
                .handle()
                .cmd_fill_buffer(self.command_buffer, buffer, 0, vk::WHOLE_SIZE, 0)
        };
    }

    // Does nothing when the device cannot switch polygon modes dynamically.
    pub fn set_wireframe(&self, wireframe: bool) {
        let Some(extended_dynamic_state3) = self.device.extended_dynamic_state3() else {
            return;
        };
        let polygon_mode = if wireframe {
            vk::PolygonMode::LINE
        } else {
            vk::PolygonMode::FILL
        };

        unsafe { extended_dynamic_state3.cmd_set_polygon_mode(self.command_buffer, polygon_mode) };
    }

    pub fn copy_image_to_buffer(&self, image: vk::Image, extent: vk::Extent2D, buffer: vk::Buffer) {
        let region = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .image_extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            });

        unsafe {
            self.device.handle().cmd_copy_image_to_buffer(
                self.command_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                buffer,
                &[region],
            );
        }
    }

    fn bind(
        &self,
        bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
        layout: vk::PipelineLayout,
        descriptor_set: vk::DescriptorSet,
    ) {
        let device = self.device.handle();

        unsafe {
            device.cmd_bind_pipeline(self.command_buffer, bind_point, pipeline);
            device.cmd_bind_descriptor_sets(self.command_buffer, bind_point, layout, 0, &[descriptor_set], &[]);
        }
    }

    fn push_constants(&self, layout: vk::PipelineLayout, stages: vk::ShaderStageFlags, data: &impl GpuData) {
        unsafe {
            self.device
                .handle()
                .cmd_push_constants(self.command_buffer, layout, stages, 0, data.as_bytes())
        };
    }
}
