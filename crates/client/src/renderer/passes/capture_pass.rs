mod capture_inputs;

pub(in crate::renderer) use capture_inputs::CaptureInputs;

use render_graph::{BufferUsage, PassContext, PassDeclaration, PassError, RenderPass};

use crate::renderer::frame_context::FrameContext;

// Copies the presented image into the capture buffer, on frames that asked for one.
pub(in crate::renderer) struct CapturePass;

impl<'frame> RenderPass<FrameContext<'frame>> for CapturePass {
    const NAME: &'static str = "capture";

    type Inputs = CaptureInputs;
    type Resources = CaptureInputs;

    fn declare(&self, pass: &mut PassDeclaration, inputs: CaptureInputs) -> CaptureInputs {
        pass.transfer_source(inputs.output);
        pass.buffer(inputs.capture, BufferUsage::TransferDestination);

        inputs
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        inputs: &CaptureInputs,
    ) -> Result<(), PassError> {
        frame.recorder.copy_image_to_buffer(
            pass.image(inputs.output),
            pass.extent(inputs.output),
            pass.buffer(inputs.capture),
        );

        Ok(())
    }
}
