use anyhow::Context;
use ash::vk::{self, Handle};
use xui::{ATLAS_SIZE, DrawList};

use crate::atlas_image::AtlasImage;
use crate::frame_resources::FrameResources;
use crate::interface_pipeline::InterfacePipeline;
use crate::memory_types::MemoryTypes;
use crate::primitive_instance::PrimitiveInstance;
use crate::push_constants::PushConstants;
use crate::vulkan_functions::VulkanFunctions;
use crate::{FrameTarget, VulkanHost};

const QUAD_VERTICES: u32 = 6;

/// Records `xui` draw lists into a host's command buffers.
pub struct Renderer {
    functions: VulkanFunctions,
    memory_types: MemoryTypes,
    pipeline: InterfacePipeline,
    atlas: AtlasImage,
    frames: Vec<FrameResources>,
    instances: Vec<PrimitiveInstance>,
}

impl Renderer {
    /// Creates the pipeline, the glyph atlas and the per-frame buffers. `color_format` is the raw `VkFormat` of
    /// the images frames are recorded into; `frames_in_flight` is how many frames the host records ahead.
    ///
    /// # Safety
    ///
    /// The handles in `host` are valid and stay alive until [`Renderer::destroy`], and the device meets the
    /// requirements listed on [`VulkanHost`].
    pub unsafe fn new(host: &VulkanHost, color_format: i32, frames_in_flight: usize) -> anyhow::Result<Renderer> {
        let functions = unsafe { VulkanFunctions::load(host) }?;
        let device = &functions.device;
        let memory_types = unsafe { MemoryTypes::new(&functions) };
        let mut pipeline = unsafe {
            InterfacePipeline::new(
                device,
                vk::Format::from_raw(color_format),
                u32::try_from(frames_in_flight)?,
            )
        }?;
        let mut atlas = match unsafe { AtlasImage::new(device, &memory_types) } {
            Ok(atlas) => atlas,
            Err(error) => {
                unsafe { pipeline.destroy(device) };
                return Err(error);
            }
        };
        let mut frames = Vec::with_capacity(frames_in_flight);
        let created = unsafe { pipeline.allocate_sets(device, frames_in_flight) }.and_then(|sets| {
            for set in sets {
                frames
                    .push(unsafe { FrameResources::new(device, &memory_types, set, atlas.view(), pipeline.sampler) }?);
            }

            Ok(())
        });

        if let Err(error) = created {
            unsafe {
                for frame in &mut frames {
                    frame.destroy(device);
                }

                atlas.destroy(device);
                pipeline.destroy(device);
            }
            return Err(error);
        }

        Ok(Renderer {
            functions,
            memory_types,
            pipeline,
            atlas,
            frames,
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
        let device = &self.functions.device;
        let command_buffer = vk::CommandBuffer::from_raw(target.command_buffer);
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
            frame.write_staging(device, &self.memory_types, &pixels)?;
            self.atlas
                .record_updates(device, command_buffer, frame.staging(), &regions);
        }

        self.instances.clear();
        self.instances
            .extend(draw_list.primitives.iter().map(PrimitiveInstance::new));

        if self.instances.is_empty() {
            return Ok(());
        }

        unsafe { frame.write_instances(device, &self.memory_types, &self.instances) }?;

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
            .image_view(vk::ImageView::from_raw(target.image_view))
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
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[frame.descriptor_set()],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                push_constants.bytes(),
            );
            device.cmd_draw(command_buffer, QUAD_VERTICES, self.instances.len() as u32, 0, 0);
            device.cmd_end_rendering(command_buffer);
        }

        Ok(())
    }

    /// Destroys every Vulkan object the renderer created.
    ///
    /// # Safety
    ///
    /// The GPU has finished every frame recorded with this renderer, and the renderer is not used afterward.
    pub unsafe fn destroy(&mut self) {
        let device = &self.functions.device;

        unsafe {
            for frame in &mut self.frames {
                frame.destroy(device);
            }

            self.atlas.destroy(device);
            self.pipeline.destroy(device);
        }
    }
}
