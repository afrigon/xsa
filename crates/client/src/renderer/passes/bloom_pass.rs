mod bloom_resources;
mod bloom_step;

pub(in crate::renderer) use bloom_resources::BloomResources;

use ash::vk;
use render_graph::{
    GraphImageDescription, ImageHandle, ImageSize, PassContext, PassDeclaration, PassError, RenderPass, Stage,
};

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::BloomPushConstants;
use crate::vulkan::ComputePipeline;
use bloom_step::BloomStep;

// Must match bloomWeightSum in tonemap.slang.
const BLOOM_LEVELS: u32 = 6;
const BLOOM_FORMAT: vk::Format = vk::Format::B10G11R11_UFLOAT_PACK32;
const OUTPUT_DIVISOR: u32 = 2;
const WORKGROUP_SIZE: u32 = 8;

// Level 0 is half the window's resolution; each further level halves again.
pub(in crate::renderer) struct BloomPass {
    downsample: ComputePipeline,
    upsample: ComputePipeline,
}

impl BloomPass {
    pub fn new(gpu: &GpuContext, binaries: &ShaderBinaries) -> anyhow::Result<BloomPass> {
        let pipeline = |spirv: &[u8]| {
            ComputePipeline::new(
                &gpu.device,
                spirv,
                size_of::<BloomPushConstants>() as u32,
                &[gpu.bindless.layout()],
            )
        };
        let downsample = pipeline(&binaries.bloom_downsample)?;
        let upsample = pipeline(&binaries.bloom_upsample)?;

        Ok(BloomPass { downsample, upsample })
    }

    pub unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            self.downsample.destroy(&gpu.device);
            self.upsample.destroy(&gpu.device);
        }
    }

    fn dispatch(
        &self,
        frame: &FrameContext,
        pass: &PassContext,
        pipeline: &ComputePipeline,
        bloom: ImageHandle,
        step: &BloomStep,
    ) -> anyhow::Result<()> {
        let recorder = &frame.recorder;
        recorder.bind_compute(pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            pipeline,
            &BloomPushConstants {
                source_texture: step.source_texture,
                source_level: step.source_level,
                target_image: frame.storage_image(pass, bloom, step.target_level)?,
                karis_average: u32::from(step.karis_average),
            },
        );
        recorder.dispatch(pass.extent(bloom.level(step.target_level)), WORKGROUP_SIZE);

        Ok(())
    }
}

impl<'frame> RenderPass<FrameContext<'frame>> for BloomPass {
    const NAME: &'static str = "bloom";

    type Inputs = ImageHandle;
    type Resources = BloomResources;

    fn declare(&self, pass: &mut PassDeclaration, scene_color: ImageHandle) -> BloomResources {
        let bloom = pass.create_image(GraphImageDescription {
            name: "bloom",
            format: BLOOM_FORMAT,
            size: ImageSize::OutputDivided(OUTPUT_DIVISOR),
            mip_levels: BLOOM_LEVELS,
        });
        pass.sampled(scene_color, Stage::Compute);
        pass.storage_write(bloom.level(0), Stage::Compute);

        for level in 1..BLOOM_LEVELS {
            pass.next_step();
            pass.sampled(bloom.level(level - 1), Stage::Compute);
            pass.storage_write(bloom.level(level), Stage::Compute);
        }

        for level in (0..BLOOM_LEVELS - 1).rev() {
            pass.next_step();
            pass.sampled(bloom.level(level + 1), Stage::Compute);
            pass.storage_read_write(bloom.level(level), Stage::Compute);
        }

        BloomResources { scene_color, bloom }
    }

    fn record(
        &mut self,
        frame: &FrameContext<'frame>,
        pass: &PassContext,
        resources: &BloomResources,
    ) -> Result<(), PassError> {
        let bloom = resources.bloom;
        let bloom_texture = frame.texture(pass, bloom)?;
        let first = BloomStep {
            source_texture: frame.texture(pass, resources.scene_color)?,
            source_level: 0,
            target_level: 0,
            karis_average: true,
        };
        self.dispatch(frame, pass, &self.downsample, bloom, &first)?;

        for level in 1..BLOOM_LEVELS {
            pass.next_step();
            let step = BloomStep {
                source_texture: bloom_texture,
                source_level: level - 1,
                target_level: level,
                karis_average: false,
            };
            self.dispatch(frame, pass, &self.downsample, bloom, &step)?;
        }

        for level in (0..BLOOM_LEVELS - 1).rev() {
            pass.next_step();
            let step = BloomStep {
                source_texture: bloom_texture,
                source_level: level + 1,
                target_level: level,
                karis_average: false,
            };
            self.dispatch(frame, pass, &self.upsample, bloom, &step)?;
        }

        Ok(())
    }
}
