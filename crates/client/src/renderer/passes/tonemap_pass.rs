mod tonemap_inputs;

pub(in crate::renderer) use tonemap_inputs::TonemapInputs;

use ash::vk;
use render_graph::{Attachment, PassContext, PassDeclaration, PassError, RenderPass, Stage};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::TonemapPushConstants;
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

// tonemap.slang reads the bloom texture only when the bloom strength is positive.
const UNSAMPLED_BLOOM_TEXTURE: u32 = 0;
const NO_BLOOM_STRENGTH: f32 = 0.0;

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
                push_constant_size: size_of::<TonemapPushConstants>() as u32,
                descriptor_set_layouts: &[gpu.bindless.layout()],
            },
        )?;

        Ok(TonemapPass { pipeline })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for TonemapPass {
    const NAME: &'static str = "tonemap";

    type Inputs = TonemapInputs;
    type Resources = TonemapInputs;

    fn declare(&self, pass: &mut PassDeclaration, inputs: TonemapInputs) -> TonemapInputs {
        pass.sampled(inputs.scene_color, Stage::Fragment);

        if let Some(bloom) = inputs.bloom {
            pass.sampled(bloom, Stage::Fragment);
        }

        pass.color_attachment(inputs.output, Attachment::DontCare);

        inputs
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        inputs: &TonemapInputs,
    ) -> Result<(), PassError> {
        let (bloom_texture, bloom_strength) = match inputs.bloom {
            Some(bloom) => (frame.texture(pass, bloom)?, frame.render.bloom.strength),
            None => (UNSAMPLED_BLOOM_TEXTURE, NO_BLOOM_STRENGTH),
        };
        let recorder = &frame.recorder;
        pass.begin_rendering();
        recorder.bind_graphics(&self.pipeline, frame.descriptor_set);
        recorder.push_graphics_constants(
            &self.pipeline,
            &TonemapPushConstants {
                frame: frame.frame.frame_data.device_address(),
                scene_color_texture: frame.texture(pass, inputs.scene_color)?,
                bloom_texture,
                bloom_strength,
                padding: 0,
            },
        );
        recorder.draw_fullscreen();
        pass.end_rendering();

        Ok(())
    }
}
