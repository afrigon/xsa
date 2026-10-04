use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::render_pass::RenderPass;

// Copies the presented image into the frame's capture buffer, on frames that asked for one.
pub(in crate::renderer) struct CapturePass;

impl RenderPass for CapturePass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()> {
        let Some(buffer) = frame.capture else {
            return Ok(());
        };

        frame.recorder.color_attachment_to_transfer_source(frame.output_image);
        frame
            .recorder
            .copy_image_to_buffer(frame.output_image, frame.output_extent, buffer);
        frame.recorder.transfer_source_to_color_attachment(frame.output_image);

        Ok(())
    }

    unsafe fn destroy(&mut self, _gpu: &mut GpuContext) {}
}
