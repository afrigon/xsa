use render_graph_vulkan::{ImageHandle, PassContext, PassDeclaration, PassError, RenderPass, Stage};

use crate::renderer::frame_context::FrameContext;

// Converts the captured frame into the video encoder's next input.
pub(in crate::renderer) struct VideoConversionPass;

impl<'frame> RenderPass<FrameContext<'frame>> for VideoConversionPass {
    const NAME: &'static str = "video conversion";

    type Inputs = ImageHandle;
    type Resources = ImageHandle;

    fn declare(&self, pass: &mut PassDeclaration, frame: ImageHandle) -> ImageHandle {
        // It writes the encoder's input, which the graph does not see.
        pass.keep();
        pass.sampled(frame, Stage::Compute);

        frame
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        image: &ImageHandle,
    ) -> Result<(), PassError> {
        let Some(video) = frame.video else {
            return Ok(());
        };
        video.record_conversion(
            frame.frame.command_buffer,
            frame.frame_slot,
            pass.view(*image),
            pass.sampled_layout(*image),
            pass.extent(*image),
        )?;

        Ok(())
    }
}
