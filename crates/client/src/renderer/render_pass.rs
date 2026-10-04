use super::frame_context::FrameContext;
use super::gpu_context::GpuContext;
use super::render_targets::RenderTargets;

pub(super) trait RenderPass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()>;

    fn resize(&mut self, _gpu: &mut GpuContext, _targets: &RenderTargets) -> anyhow::Result<()> {
        Ok(())
    }

    // Safety: only once the GPU has finished every frame that used the pass.
    unsafe fn destroy(&mut self, gpu: &mut GpuContext);
}
