use anyhow::Context;
use ash::vk;
use memory_vulkan::MemoryContext;
use xui::{ATLAS_SIZE, DrawList};

use crate::FrameTarget;
use crate::atlas_image::AtlasImage;
use crate::frame_resources::FrameResources;
use crate::interface_pipeline::InterfacePipeline;
use crate::primitive_instance::PrimitiveInstance;
use crate::push_constants::PushConstants;

const QUAD_VERTICES: u32 = 6;

/// Records `xui` draw lists into a host's command buffers. Its Vulkan objects are destroyed through the host's
/// deletion queue once dropped.
pub struct Renderer {
    memory: MemoryContext,
    frames: Vec<FrameResources>,
    atlas: AtlasImage,
    pipeline: InterfacePipeline,
    instances: Vec<PrimitiveInstance>,
}

impl Renderer {
    /// Creates the pipeline, the glyph atlas and the per-frame buffers from `memory`, whose device has Vulkan 1.3's
    /// dynamic rendering and synchronization2 enabled. `color_format` is the format of the images frames are
    /// recorded into; `frames_in_flight` is how many frames the host records ahead.
    pub fn new(memory: &MemoryContext, color_format: vk::Format, frames_in_flight: usize) -> anyhow::Result<Renderer> {
        let pipeline = InterfacePipeline::new(memory, color_format, u32::try_from(frames_in_flight)?)?;
        let atlas = AtlasImage::new(memory)?;
        let frames = pipeline
            .allocate_sets(memory.device(), frames_in_flight)?
            .into_iter()
            .map(|set| FrameResources::new(memory, set, atlas.view(), pipeline.sampler()))
            .collect::<anyhow::Result<_>>()?;

        Ok(Renderer {
            memory: memory.clone(),
            frames,
            atlas,
            pipeline,
            instances: Vec::new(),
        })
    }

    /// Uploads the draw list's new glyphs and draws its primitives over the target image, in order.
    ///
    /// # Safety
    ///
    /// The target's handles are valid. The image is in `COLOR_ATTACHMENT_OPTIMAL` layout, was last written as a
    /// color attachment, and is left in that layout. The GPU has finished the frame previously recorded with the
    /// same `frame_slot`.
    pub unsafe fn record(&mut self, target: &FrameTarget, draw_list: &DrawList) -> anyhow::Result<()> {
        let device = self.memory.device();
        let command_buffer = target.command_buffer;
        let frame = self
            .frames
            .get_mut(target.frame_slot)
            .with_context(|| format!("frame slot {} is out of range", target.frame_slot))?;
        let mut pixels = Vec::new();
        let mut regions = Vec::with_capacity(draw_list.atlas_updates.len());

        for update in &draw_list.atlas_updates {
            regions.push(
                vk::BufferImageCopy::default()
                    .buffer_offset(pixels.len() as u64)
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_offset(vk::Offset3D {
                        x: update.region.x as i32,
                        y: update.region.y as i32,
                        z: 0,
                    })
                    .image_extent(vk::Extent3D {
                        width: update.region.width,
                        height: update.region.height,
                        depth: 1,
                    }),
            );
            pixels.extend_from_slice(&update.pixels);
        }

        unsafe {
            frame.write_staging(&self.memory, &pixels)?;
            self.atlas
                .record_updates(device, command_buffer, frame.staging(), &regions);
        }

        self.instances.clear();
        self.instances
            .extend(draw_list.primitives.iter().map(PrimitiveInstance::new));

        if self.instances.is_empty() {
            return Ok(());
        }

        frame.write_instances(&self.memory, &self.instances)?;

        let attachment_barriers = [vk::MemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)];
        let extent = vk::Extent2D {
            width: target.width,
            height: target.height,
        };
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(target.image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE)];
        let rendering = vk::RenderingInfo::default()
            .render_area(extent.into())
            .layer_count(1)
            .color_attachments(&color_attachments);
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: target.width as f32,
            height: target.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let push_constants = PushConstants {
            viewport_size: [target.width as f32, target.height as f32],
            atlas_size: [ATLAS_SIZE as f32; 2],
        };

        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().memory_barriers(&attachment_barriers),
            );
            device.cmd_begin_rendering(command_buffer, &rendering);
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            device.cmd_set_scissor(command_buffer, 0, &[extent.into()]);
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline(),
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout(),
                0,
                &[frame.descriptor_set()],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                self.pipeline.layout(),
                vk::ShaderStageFlags::VERTEX,
                0,
                push_constants.bytes(),
            );
            device.cmd_draw(command_buffer, QUAD_VERTICES, self.instances.len() as u32, 0, 0);
            device.cmd_end_rendering(command_buffer);
        }

        Ok(())
    }
}
