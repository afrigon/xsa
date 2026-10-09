use ash::vk::{self, Handle};
use render_graph::{Attachment, ImageHandle, PassContext, PassDeclaration, PassError, RenderPass};
use xui_vulkan::{FrameTarget, VulkanHost};

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
        let host = VulkanHost {
            get_instance_proc_addr: gpu.instance.entry().static_fn().get_instance_proc_addr as *const _,
            instance: gpu.instance.handle().handle().as_raw(),
            physical_device: gpu.device.physical_device().as_raw(),
            device: gpu.device.handle().handle().as_raw(),
        };
        let renderer = unsafe { xui_vulkan::Renderer::new(&host, output_format.as_raw(), frames_in_flight) }?;

        Ok(UserInterfacePass { renderer })
    }

    pub unsafe fn destroy(&mut self) {
        unsafe { self.renderer.destroy() };
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for UserInterfacePass {
    const NAME: &'static str = "user interface";

    type Inputs = ImageHandle;
    type Resources = ImageHandle;

    fn declare(&self, pass: &mut PassDeclaration, output: ImageHandle) -> ImageHandle {
        pass.color_attachment(output, Attachment::Load);

        output
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        output: &ImageHandle,
    ) -> Result<(), PassError> {
        let extent = pass.extent(*output);
        let target = FrameTarget {
            command_buffer: frame.frame.command_buffer.as_raw(),
            image_view: pass.view(*output).as_raw(),
            width: extent.width,
            height: extent.height,
            frame_slot: frame.frame_slot,
        };

        unsafe { self.renderer.record(&target, frame.user_interface) }?;

        Ok(())
    }
}
