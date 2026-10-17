use ash::vk;
use render_graph_vulkan::{Attachment, ImportedImageHandle, PassContext, PassDeclaration, PassError, RenderPass};
use xui_vulkan::FrameTarget;

use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;

pub(in crate::renderer) struct UserInterfacePass {
    renderer: xui_vulkan::Renderer,
}

impl UserInterfacePass {
    pub fn new(
        gpu: &GpuContext,
        output_format: vk::Format,
        frames_in_flight: usize,
    ) -> anyhow::Result<UserInterfacePass> {
        let renderer = xui_vulkan::Renderer::new(&gpu.memory, output_format, frames_in_flight)?;

        Ok(UserInterfacePass { renderer })
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for UserInterfacePass {
    const NAME: &'static str = "user interface";

    type Inputs = ImportedImageHandle;
    type Resources = ImportedImageHandle;

    fn declare(&self, pass: &mut PassDeclaration, output: ImportedImageHandle) -> ImportedImageHandle {
        pass.color_attachment(output, Attachment::Load);

        output
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        output: &ImportedImageHandle,
    ) -> Result<(), PassError> {
        let extent = pass.extent(*output);
        let target = FrameTarget {
            command_buffer: frame.frame.command_buffer,
            image_view: pass.view(*output),
            width: extent.width,
            height: extent.height,
            frame_slot: frame.frame_slot,
        };

        unsafe { self.renderer.record(&target, frame.user_interface) }?;

        Ok(())
    }
}
