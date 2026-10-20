mod lens_inputs;
mod lens_resources;

pub(in crate::renderer) use lens_inputs::LensInputs;
pub(in crate::renderer) use lens_resources::LensResources;

use ash::vk;
use render_graph_vulkan::{
    Attachment, GraphImageDescription, ImageSize, PassContext, PassDeclaration, PassError, RenderPass,
};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::LensPushConstants;
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

const LENS_FORMAT: vk::Format = vk::Format::R32G32B32A32_SFLOAT;
const CLEAR: [f32; 4] = [0.0; 4];

// Draws what the camera's lens adds to the image around each light source, in exposed luminance like the scene.
pub(in crate::renderer) struct LensPass {
    light_pipeline: GraphicsPipeline,
    ghosts_pipeline: GraphicsPipeline,
}

impl LensPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<LensPass> {
        Ok(LensPass {
            light_pipeline: LensPass::pipeline(gpu, &binaries.lens_light)?,
            ghosts_pipeline: LensPass::pipeline(gpu, &binaries.lens_ghosts)?,
        })
    }

    fn pipeline(gpu: &GpuContext, spirv: &[u8]) -> anyhow::Result<GraphicsPipeline> {
        GraphicsPipeline::new(
            &gpu.memory,
            &gpu.device,
            &GraphicsPipelineDescription {
                spirv,
                specialization_constants: &[],
                color_formats: &[LENS_FORMAT],
                depth_format: None,
                depth_compare_op: vk::CompareOp::ALWAYS,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: true,
                wireframe: false,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<LensPushConstants>() as u32,
                descriptor_set_layouts: &[gpu.bindless.layout()],
            },
        )
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for LensPass {
    const NAME: &'static str = "lens";

    type Inputs = LensInputs;
    type Resources = LensResources;

    fn declare(&self, pass: &mut PassDeclaration, inputs: LensInputs) -> LensResources {
        let lens = pass.create_image(GraphImageDescription {
            name: "lens",
            format: LENS_FORMAT,
            size: ImageSize::Output,
            mip_levels: 1,
        });
        pass.color_attachment(lens, Attachment::ClearColor(CLEAR));

        LensResources {
            lens,
            light_count: inputs.light_count,
        }
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        resources: &LensResources,
    ) -> Result<(), PassError> {
        let lens = frame.lens.expect("the lens pass is only added with a lens loaded");
        let recorder = &frame.recorder;
        let constants = LensPushConstants {
            frame: frame.frame.frame_data.device_address(),
            lights: frame.frame.projected_lights.device_address(),
            ghosts: lens.ghosts(),
            glare_intensity: lens.glare_intensity,
            glare_falloff: lens.glare_falloff,
            focal_length_pixels: frame.focal_length_pixels,
            light_count: resources.light_count,
            starburst_texture: lens.starburst_texture,
            starburst_inner: lens.starburst_inner,
            starburst_extent: lens.starburst_extent,
            ghost_count: lens.ghost_count,
        };
        pass.begin_rendering();
        recorder.bind_graphics(&self.light_pipeline, frame.descriptor_set);
        recorder.push_graphics_constants(&self.light_pipeline, &constants);
        recorder.draw_fullscreen_instances(resources.light_count);

        if lens.ghost_count > 0 {
            recorder.bind_graphics(&self.ghosts_pipeline, frame.descriptor_set);
            recorder.push_graphics_constants(&self.ghosts_pipeline, &constants);
            recorder.draw_quad_instances(resources.light_count * lens.ghost_count);
        }

        pass.end_rendering();

        Ok(())
    }
}
