use ash::vk;
use render_graph_vulkan::{Attachment, PassContext, PassDeclaration, PassError, RenderPass};

use super::ForwardResources;
use super::forward_pass::{DEPTH_FORMAT, HDR_FORMAT, MOTION_FORMAT};
use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::{FrameContext, NO_ADDRESS};
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::PushConstants;
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

pub(in crate::renderer) struct SkyboxPass {
    pipeline: GraphicsPipeline,
}

impl SkyboxPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<SkyboxPass> {
        let pipeline = GraphicsPipeline::new(
            &gpu.memory,
            &gpu.device,
            &GraphicsPipelineDescription {
                spirv: &binaries.skybox,
                specialization_constants: &[],
                color_formats: &[HDR_FORMAT, MOTION_FORMAT],
                depth_format: Some(DEPTH_FORMAT),
                depth_compare_op: vk::CompareOp::GREATER_OR_EQUAL,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: false,
                wireframe: false,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<PushConstants>() as u32,
                descriptor_set_layouts: &[gpu.bindless.layout()],
            },
        )?;

        Ok(SkyboxPass { pipeline })
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for SkyboxPass {
    const NAME: &'static str = "skybox";

    type Inputs = ForwardResources;
    type Resources = ForwardResources;

    fn declare(&self, pass: &mut PassDeclaration, inputs: ForwardResources) -> ForwardResources {
        pass.color_attachment(inputs.hdr, Attachment::Load);
        pass.color_attachment(inputs.motion, Attachment::Load);
        pass.depth_attachment(inputs.depth, Attachment::Load);

        inputs
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        _inputs: &ForwardResources,
    ) -> Result<(), PassError> {
        pass.begin_rendering();

        if let Some(skybox) = frame.scene.skybox
            && frame.is_drawable(skybox)
        {
            frame.recorder.bind_graphics(&self.pipeline, frame.descriptor_set);
            frame.push_draw_constants(&self.pipeline, skybox, NO_ADDRESS);
            frame.recorder.draw_fullscreen();
        }

        pass.end_rendering();

        Ok(())
    }
}
