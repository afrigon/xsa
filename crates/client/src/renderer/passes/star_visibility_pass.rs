mod star_visibility_inputs;
mod star_visibility_resources;

pub(in crate::renderer) use star_visibility_inputs::StarVisibilityInputs;
pub(in crate::renderer) use star_visibility_resources::StarVisibilityResources;

use render_graph::{BufferUsage, GraphBufferDescription, PassContext, PassDeclaration, PassError, RenderPass, Stage};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::StarVisibilityPushConstants;
use crate::vulkan::ComputePipeline;

// Measures the fraction of each star's disk that nothing in the scene hides: one visibility per projected star.
pub(in crate::renderer) struct StarVisibilityPass {
    pipeline: ComputePipeline,
}

impl StarVisibilityPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<StarVisibilityPass> {
        let pipeline = ComputePipeline::new(
            &gpu.device,
            &binaries.star_visibility,
            size_of::<StarVisibilityPushConstants>() as u32,
            &[gpu.bindless.layout()],
        )?;

        Ok(StarVisibilityPass { pipeline })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for StarVisibilityPass {
    const NAME: &'static str = "star visibility";

    type Inputs = StarVisibilityInputs;
    type Resources = StarVisibilityResources;

    fn declare(&self, pass: &mut PassDeclaration, inputs: StarVisibilityInputs) -> StarVisibilityResources {
        let visibility = pass.create_buffer(GraphBufferDescription {
            name: "star visibility",
            size: (inputs.star_count as usize * size_of::<f32>()) as u64,
        });
        pass.sampled(inputs.depth, Stage::Compute);
        pass.buffer(visibility, BufferUsage::StorageWrite(Stage::Compute));

        StarVisibilityResources {
            depth: inputs.depth,
            visibility,
            star_count: inputs.star_count,
        }
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        resources: &StarVisibilityResources,
    ) -> Result<(), PassError> {
        let recorder = &frame.recorder;
        recorder.bind_compute(&self.pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            &self.pipeline,
            &StarVisibilityPushConstants {
                frame: frame.frame.frame_data.device_address(),
                stars: frame.frame.projected_stars.device_address(),
                visibility: pass.device_address(resources.visibility),
                depth_texture: frame.texture(pass, resources.depth),
                padding: 0,
            },
        );
        recorder.dispatch_groups(resources.star_count);

        Ok(())
    }
}
