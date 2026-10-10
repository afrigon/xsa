use std::cell::Cell;

use ash::vk;

use crate::barrier_recorder::BarrierRecorder;
use crate::compiled_pass::CompiledPass;
use crate::graph_compiler::MAX_COLOR_ATTACHMENTS;
use crate::resolved_buffer::ResolvedBuffer;
use crate::resolved_image::ResolvedImage;
use crate::{BufferHandle, BufferResource, ImageHandle, ImageId, Subresource};

const VIEWPORT_MIN_DEPTH: f32 = 0.0;
const VIEWPORT_MAX_DEPTH: f32 = 1.0;

/// What the graph gives a pass while it records: its resources' handles and the barriers between its steps.
///
/// Every resource accessor panics when no running pass uses the resource, which means this pass uses it without
/// declaring it.
pub struct PassContext<'frame> {
    recorder: &'frame BarrierRecorder<'frame>,
    pass: &'frame CompiledPass,
    step: Cell<usize>,
}

impl<'frame> PassContext<'frame> {
    pub(crate) fn new(recorder: &'frame BarrierRecorder<'frame>, pass: &'frame CompiledPass) -> PassContext<'frame> {
        let context = PassContext {
            recorder,
            pass,
            step: Cell::new(0),
        };
        context.record_barriers();

        context
    }

    pub(crate) fn recorded_steps(&self) -> u32 {
        self.step.get() as u32 + 1
    }

    /// Records the barriers the next declared step needs. Call it between steps, as many times as `next_step` was
    /// called while declaring.
    pub fn next_step(&self) {
        self.step.set(self.step.get() + 1);
        self.record_barriers();
    }

    pub fn image(&self, image: impl Into<Subresource>) -> vk::Image {
        self.resolved_image(image.into().image).image
    }

    /// The graph's id for an image this pass declared.
    pub fn image_id(&self, image: ImageHandle) -> ImageId {
        self.resolved_image(image.index)
            .id
            .expect("a used graph image is pooled")
    }

    pub fn view(&self, subresource: impl Into<Subresource>) -> vk::ImageView {
        let subresource = subresource.into();
        let resolved = self.resolved_image(subresource.image);

        match (subresource.level, resolved.id) {
            (Some(level), Some(id)) => self.recorder.image_pool.image(id).level_view(level),
            _ => resolved.view,
        }
    }

    pub fn extent(&self, subresource: impl Into<Subresource>) -> vk::Extent2D {
        let subresource = subresource.into();

        self.resolved_image(subresource.image)
            .level_extent(subresource.level.unwrap_or(0))
    }

    pub fn buffer(&self, buffer: impl Into<BufferResource>) -> vk::Buffer {
        self.resolved_buffer(buffer.into().buffer).buffer
    }

    /// The device address shaders reach a buffer this pass declared through.
    pub fn device_address(&self, buffer: BufferHandle) -> vk::DeviceAddress {
        let id = self
            .resolved_buffer(buffer.index)
            .id
            .expect("a used graph buffer is pooled");

        self.recorder.buffer_pool.buffer(id).address
    }

    /// Begins rendering to the current step's attachments, with the viewport and scissor covering them.
    pub fn begin_rendering(&self) {
        let Some(step) = self.pass.steps.get(self.step.get()) else {
            return;
        };
        let mut colors = [vk::RenderingAttachmentInfo::default(); MAX_COLOR_ATTACHMENTS];
        let mut color_count = 0;
        let mut depth = None;
        let mut extent = vk::Extent2D::default();

        for attachment in &step.attachments {
            let resolved = &self.recorder.resolved_images[attachment.image];
            let view = match resolved.id {
                Some(id) => self.recorder.image_pool.image(id).level_view(attachment.level),
                None => resolved.view,
            };
            let info = vk::RenderingAttachmentInfo::default()
                .image_view(view)
                .image_layout(attachment.layout)
                .load_op(attachment.load_op)
                .store_op(attachment.store_op)
                .clear_value(attachment.clear_value);
            extent = resolved.level_extent(attachment.level);

            if attachment.depth {
                depth = Some(info);
            } else {
                colors[color_count] = info;
                color_count += 1;
            }
        }

        let mut rendering_info = vk::RenderingInfo::default()
            .render_area(extent.into())
            .layer_count(1)
            .color_attachments(&colors[..color_count]);

        if let Some(depth) = &depth {
            rendering_info = rendering_info.depth_attachment(depth);
        }

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: VIEWPORT_MIN_DEPTH,
            max_depth: VIEWPORT_MAX_DEPTH,
        };
        let device = self.recorder.device;
        let command_buffer = self.recorder.command_buffer;

        unsafe {
            device.cmd_begin_rendering(command_buffer, &rendering_info);
            device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            device.cmd_set_scissor(command_buffer, 0, &[extent.into()]);
        }
    }

    pub fn end_rendering(&self) {
        unsafe { self.recorder.device.cmd_end_rendering(self.recorder.command_buffer) };
    }

    fn resolved_image(&self, index: usize) -> &ResolvedImage {
        let resolved = &self.recorder.resolved_images[index];

        if !resolved.used {
            panic!("{} is not used by any running pass", resolved.name);
        }

        resolved
    }

    fn resolved_buffer(&self, index: usize) -> &ResolvedBuffer {
        let resolved = &self.recorder.resolved_buffers[index];

        if !resolved.used {
            panic!("{} is not used by any running pass", resolved.name);
        }

        resolved
    }

    // A step past the declared ones records nothing; execution then reports the mismatch.
    fn record_barriers(&self) {
        if let Some(step) = self.pass.steps.get(self.step.get()) {
            self.recorder.record(&step.image_barriers, &step.buffer_barriers);
        }
    }
}
