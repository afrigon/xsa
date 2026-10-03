use ash::vk;

use super::gpu_data::GpuData;
use crate::vulkan::{Buffer, ComputePipeline, Device, GraphicsPipeline};

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const REVERSE_Z_FAR_DEPTH: f32 = 0.0;
const FULLSCREEN_VERTEX_COUNT: u32 = 3;
const GRAPHICS_STAGES: vk::ShaderStageFlags =
    vk::ShaderStageFlags::from_raw(vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw());
const DEPTH_TESTS: vk::PipelineStageFlags2 = vk::PipelineStageFlags2::from_raw(
    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS.as_raw() | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS.as_raw(),
);
const COMPUTE_IMAGE_ACCESS: vk::AccessFlags2 = vk::AccessFlags2::from_raw(
    vk::AccessFlags2::SHADER_STORAGE_READ.as_raw()
        | vk::AccessFlags2::SHADER_STORAGE_WRITE.as_raw()
        | vk::AccessFlags2::SHADER_SAMPLED_READ.as_raw(),
);

pub(super) struct CommandRecorder<'a> {
    device: &'a Device,
    command_buffer: vk::CommandBuffer,
}

struct ImageTransition {
    image: vk::Image,
    aspect: vk::ImageAspectFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    source_stage: vk::PipelineStageFlags2,
    source_access: vk::AccessFlags2,
    destination_stage: vk::PipelineStageFlags2,
    destination_access: vk::AccessFlags2,
}

struct BufferDependency {
    buffer: vk::Buffer,
    source_stage: vk::PipelineStageFlags2,
    source_access: vk::AccessFlags2,
    destination_stage: vk::PipelineStageFlags2,
    destination_access: vk::AccessFlags2,
}

impl<'a> CommandRecorder<'a> {
    pub fn new(device: &'a Device, command_buffer: vk::CommandBuffer) -> CommandRecorder<'a> {
        CommandRecorder { device, command_buffer }
    }

    pub fn begin_rendering(&self, color: vk::ImageView, depth: Option<vk::ImageView>, extent: vk::Extent2D) {
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(color)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue { float32: CLEAR_COLOR },
            })];
        let depth_attachment = depth.map(|view| {
            vk::RenderingAttachmentInfo::default()
                .image_view(view)
                .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: REVERSE_Z_FAR_DEPTH,
                        stencil: 0,
                    },
                })
        });
        let mut rendering_info = vk::RenderingInfo::default()
            .render_area(extent.into())
            .layer_count(1)
            .color_attachments(&color_attachments);

        if let Some(depth_attachment) = &depth_attachment {
            rendering_info = rendering_info.depth_attachment(depth_attachment);
        }

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let device = self.device.handle();

        unsafe {
            device.cmd_begin_rendering(self.command_buffer, &rendering_info);
            device.cmd_set_viewport(self.command_buffer, 0, &[viewport]);
            device.cmd_set_scissor(self.command_buffer, 0, &[extent.into()]);
        }
    }

    pub fn end_rendering(&self) {
        unsafe { self.device.handle().cmd_end_rendering(self.command_buffer) };
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

    pub fn clear_buffer(&self, buffer: &Buffer) {
        unsafe {
            self.device
                .handle()
                .cmd_fill_buffer(self.command_buffer, buffer.handle(), 0, vk::WHOLE_SIZE, 0)
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

    pub fn to_color_attachment(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            source_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            source_access: vk::AccessFlags2::NONE,
            destination_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            destination_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        });
    }

    pub fn sampled_to_color_attachment(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            source_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER,
            source_access: vk::AccessFlags2::NONE,
            destination_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            destination_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        });
    }

    pub fn color_attachment_to_sampled(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            source_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            source_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            destination_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
        });
    }

    pub fn to_depth_attachment(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::DEPTH,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            source_stage: DEPTH_TESTS,
            source_access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            destination_stage: DEPTH_TESTS,
            destination_access: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        });
    }

    pub fn to_present(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            source_stage: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            source_access: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            destination_stage: vk::PipelineStageFlags2::NONE,
            destination_access: vk::AccessFlags2::NONE,
        });
    }

    pub fn bloom_start(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::GENERAL,
            source_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            source_access: vk::AccessFlags2::NONE,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_access: COMPUTE_IMAGE_ACCESS,
        });
    }

    pub fn bloom_between_passes(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::GENERAL,
            new_layout: vk::ImageLayout::GENERAL,
            source_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            source_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_access: COMPUTE_IMAGE_ACCESS,
        });
    }

    pub fn bloom_to_fragment(&self, image: vk::Image) {
        self.transition(&ImageTransition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::GENERAL,
            new_layout: vk::ImageLayout::GENERAL,
            source_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            source_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            destination_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
        });
    }

    pub fn cleared_to_compute(&self, buffer: &Buffer) {
        self.buffer_dependency(&BufferDependency {
            buffer: buffer.handle(),
            source_stage: vk::PipelineStageFlags2::CLEAR,
            source_access: vk::AccessFlags2::TRANSFER_WRITE,
            destination_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            destination_access: vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
        });
    }

    pub fn compute_to_host(&self, buffer: &Buffer) {
        self.buffer_dependency(&BufferDependency {
            buffer: buffer.handle(),
            source_stage: vk::PipelineStageFlags2::COMPUTE_SHADER,
            source_access: vk::AccessFlags2::SHADER_STORAGE_WRITE,
            destination_stage: vk::PipelineStageFlags2::HOST,
            destination_access: vk::AccessFlags2::HOST_READ,
        });
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

    fn transition(&self, transition: &ImageTransition) {
        let barriers = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(transition.source_stage)
            .src_access_mask(transition.source_access)
            .dst_stage_mask(transition.destination_stage)
            .dst_access_mask(transition.destination_access)
            .old_layout(transition.old_layout)
            .new_layout(transition.new_layout)
            .image(transition.image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(transition.aspect)
                    .level_count(vk::REMAINING_MIP_LEVELS)
                    .layer_count(1),
            )];
        let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);

        unsafe {
            self.device
                .handle()
                .cmd_pipeline_barrier2(self.command_buffer, &dependency_info)
        };
    }

    fn buffer_dependency(&self, dependency: &BufferDependency) {
        let barriers = [vk::BufferMemoryBarrier2::default()
            .src_stage_mask(dependency.source_stage)
            .src_access_mask(dependency.source_access)
            .dst_stage_mask(dependency.destination_stage)
            .dst_access_mask(dependency.destination_access)
            .buffer(dependency.buffer)
            .size(vk::WHOLE_SIZE)];
        let dependency_info = vk::DependencyInfo::default().buffer_memory_barriers(&barriers);

        unsafe {
            self.device
                .handle()
                .cmd_pipeline_barrier2(self.command_buffer, &dependency_info)
        };
    }
}
