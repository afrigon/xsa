use ash::vk::{self, Handle};
use xui_vulkan::{FrameTarget, VulkanHost};

use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::render_pass::RenderPass;

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
}

impl RenderPass for UserInterfacePass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()> {
        let target = FrameTarget {
            command_buffer: frame.frame.command_buffer.as_raw(),
            image_view: frame.output_view.as_raw(),
            width: frame.output_extent.width,
            height: frame.output_extent.height,
            frame_slot: frame.frame_slot,
        };

        unsafe { self.renderer.record(&target, frame.user_interface) }
    }

    unsafe fn destroy(&mut self, _gpu: &mut GpuContext) {
        unsafe { self.renderer.destroy() };
    }
}
