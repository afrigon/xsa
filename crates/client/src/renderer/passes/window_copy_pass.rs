mod window_copy_inputs;

pub(in crate::renderer) use window_copy_inputs::WindowCopyInputs;

use ash::vk;
use render_graph_vulkan::{Attachment, PassContext, PassDeclaration, PassError, RenderPass, Stage};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::CopyPushConstants;
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

// Shows a frame captured into an image of its own in the window: the window's images can only be drawn to.
pub(in crate::renderer) struct WindowCopyPass {
    pipeline: GraphicsPipeline,
}

impl WindowCopyPass {
    pub fn new(
        gpu: &GpuContext,
        binaries: &ShaderBinaries,
        output_format: vk::Format,
    ) -> anyhow::Result<WindowCopyPass> {
        let pipeline = GraphicsPipeline::new(
            &gpu.memory,
            &gpu.device,
            &GraphicsPipelineDescription {
                spirv: &binaries.copy,
                specialization_constants: &[],
                color_formats: &[output_format],
                depth_format: None,
                depth_compare_op: vk::CompareOp::ALWAYS,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: false,
                wireframe: false,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<CopyPushConstants>() as u32,
                descriptor_set_layouts: &[gpu.bindless.layout()],
            },
        )?;

        Ok(WindowCopyPass { pipeline })
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for WindowCopyPass {
    const NAME: &'static str = "window copy";

    type Inputs = WindowCopyInputs;
    type Resources = WindowCopyInputs;

    fn declare(&self, pass: &mut PassDeclaration, inputs: WindowCopyInputs) -> WindowCopyInputs {
        pass.sampled(inputs.source, Stage::Fragment);
        pass.color_attachment(inputs.window, Attachment::DontCare);

        inputs
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        inputs: &WindowCopyInputs,
    ) -> Result<(), PassError> {
        let recorder = &frame.recorder;
        pass.begin_rendering();
        recorder.bind_graphics(&self.pipeline, frame.descriptor_set);
        recorder.push_graphics_constants(
            &self.pipeline,
            &CopyPushConstants {
                source_texture: frame.texture(pass, inputs.source),
            },
        );
        recorder.draw_fullscreen();
        pass.end_rendering();

        Ok(())
    }
}
