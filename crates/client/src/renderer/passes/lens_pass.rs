mod lens_inputs;
mod lens_resources;

pub(in crate::renderer) use lens_inputs::LensInputs;
pub(in crate::renderer) use lens_resources::LensResources;

use ash::vk;
use render_graph::{
    Attachment, BufferUsage, GraphImageDescription, ImageSize, PassContext, PassDeclaration, PassError, RenderPass,
    Stage,
};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::LensPushConstants;
use crate::vulkan::{GraphicsPipeline, GraphicsPipelineDescription};

const LENS_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const CLEAR: [f32; 4] = [0.0; 4];

// Draws what the camera's lens adds to the image around each star, in exposed luminance like the scene.
pub(in crate::renderer) struct LensPass {
    pipeline: GraphicsPipeline,
}

impl LensPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<LensPass> {
        let pipeline = GraphicsPipeline::new(
            &gpu.device,
            &GraphicsPipelineDescription {
                spirv: &binaries.lens,
                color_formats: &[LENS_FORMAT],
                depth_format: None,
                depth_compare_op: vk::CompareOp::ALWAYS,
                depth_write: false,
                cull_mode: vk::CullModeFlags::NONE,
                additive_blend: true,
                vertex_bindings: &[],
                vertex_attributes: &[],
                push_constant_size: size_of::<LensPushConstants>() as u32,
                descriptor_set_layouts: &[gpu.bindless.layout()],
            },
        )?;

        Ok(LensPass { pipeline })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
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
        pass.buffer(inputs.visibility, BufferUsage::StorageRead(Stage::Fragment));
        pass.color_attachment(lens, Attachment::ClearColor(CLEAR));

        LensResources {
            lens,
            visibility: inputs.visibility,
            star_count: inputs.star_count,
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
        pass.begin_rendering();
        recorder.bind_graphics(&self.pipeline, frame.descriptor_set);
        recorder.push_graphics_constants(
            &self.pipeline,
            &LensPushConstants {
                frame: frame.frame.frame_data.device_address(),
                stars: frame.frame.projected_stars.device_address(),
                visibility: pass.device_address(resources.visibility),
                ghosts: lens.ghosts(),
                glare_intensity: lens.glare_intensity,
                glare_falloff: lens.glare_falloff,
                focal_length_pixels: frame.focal_length_pixels,
                star_count: resources.star_count,
                spikes_intensity: lens.spikes_intensity,
                spikes_length: lens.spikes_length,
                spikes_width: lens.spikes_width,
                spikes_dispersion: lens.spikes_dispersion,
                spikes_texture: lens.spikes_texture,
                blades: lens.blades,
                rotation: lens.rotation,
                ghost_count: lens.ghost_count,
            },
        );
        recorder.draw_fullscreen_instances(resources.star_count);
        pass.end_rendering();

        Ok(())
    }
}
