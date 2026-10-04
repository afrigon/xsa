use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::HistogramPushConstants;
use crate::renderer::render_pass::RenderPass;
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
}

impl RenderPass for HistogramPass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()> {
        let recorder = &frame.recorder;
        let histogram = &frame.frame.histogram;
        recorder.clear_buffer(histogram);
        recorder.cleared_to_compute(histogram);
        recorder.bind_compute(&self.pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            &self.pipeline,
            &HistogramPushConstants {
                frame: frame.frame.frame_data.device_address(),
                histogram: histogram.device_address(),
            },
        );
        recorder.dispatch(frame.targets.extent, TILE_SIZE);
        recorder.compute_to_host(histogram);

        Ok(())
    }

    unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe { self.pipeline.destroy(&gpu.device) };
    }
}
