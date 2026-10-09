mod histogram_inputs;

pub(in crate::renderer) use histogram_inputs::HistogramInputs;

use render_graph::{BufferUsage, PassContext, PassDeclaration, PassError, RenderPass, Stage};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::HistogramPushConstants;
use crate::vulkan::ComputePipeline;

// Must match the workgroup size in histogram.slang.
const TILE_SIZE: u32 = 16;

pub(in crate::renderer) struct HistogramPass {
    pipeline: ComputePipeline,
}

impl HistogramPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<HistogramPass> {
        let pipeline = ComputePipeline::new(
            &gpu.device,
            &binaries.histogram,
            size_of::<HistogramPushConstants>() as u32,
            &[gpu.bindless.layout()],
        )?;

        Ok(HistogramPass { pipeline })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for HistogramPass {
    const NAME: &'static str = "histogram";

    type Inputs = HistogramInputs;
    type Resources = HistogramInputs;

    fn declare(&self, pass: &mut PassDeclaration, inputs: HistogramInputs) -> HistogramInputs {
        pass.buffer(inputs.histogram, BufferUsage::TransferDestination);
        pass.next_step();
        pass.sampled(inputs.scene_color, Stage::Compute);
        pass.buffer(inputs.histogram, BufferUsage::StorageReadWrite(Stage::Compute));

        inputs
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        inputs: &HistogramInputs,
    ) -> Result<(), PassError> {
        let recorder = &frame.recorder;
        recorder.clear_buffer(pass.buffer(inputs.histogram));
        pass.next_step();
        recorder.bind_compute(&self.pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            &self.pipeline,
            &HistogramPushConstants {
                frame: frame.frame.frame_data.device_address(),
                histogram: frame.frame.histogram.device_address(),
                scene_color_texture: frame.texture(pass, inputs.scene_color)?,
                padding: 0,
            },
        );
        recorder.dispatch(pass.extent(inputs.scene_color), TILE_SIZE);

        Ok(())
    }
}
