use ash::vk;

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::PushConstants;
use crate::renderer::render_pass::RenderPass;
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

pub(in crate::renderer) struct TonemapPass {
    pipeline: GraphicsPipeline,
}

impl TonemapPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries, output_format: vk::Format) -> anyhow::Result<TonemapPass> {
        let pipeline = GraphicsPipeline::new(
            &gpu.device,
            &GraphicsPipelineDescription {
                spirv: &binaries.tonemap,
                color_formats: &[output_format],
                depth_format: None,
                depth_compare_op: vk::CompareOp::ALWAYS,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: false,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<PushConstants>() as u32,
                descriptor_set_layouts: &[gpu.bindless.layout()],
            },
        )?;

        Ok(TonemapPass { pipeline })
    }
}

impl RenderPass for TonemapPass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()> {
        let recorder = &frame.recorder;
        recorder.begin_rendering(&[frame.output_view], None, frame.targets.extent);
        recorder.bind_graphics(&self.pipeline, frame.descriptor_set);
        frame.push_draw_constants(&self.pipeline, 0, 0);
        recorder.draw_fullscreen();
        recorder.end_rendering();

        Ok(())
    }

    unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
    }
}
