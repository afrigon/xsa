mod bloom_resources;
mod bloom_step;

use ash::vk;

use crate::renderer::ShaderBinaries;
use crate::renderer::frame_context::FrameContext;
use crate::renderer::gpu_context::GpuContext;
use crate::renderer::gpu_data::BloomPushConstants;
use crate::renderer::render_pass::RenderPass;
use crate::renderer::render_targets::RenderTargets;
use crate::vulkan::ComputePipeline;
use bloom_resources::BloomResources;
use bloom_step::BloomStep;

// Must match bloomWeightSum in tonemap.slang.
const BLOOM_LEVELS: u32 = 6;
const WORKGROUP_SIZE: u32 = 8;

// Level 0 is half the window's resolution; each further level halves again.
pub(in crate::renderer) struct BloomPass {
    downsample: ComputePipeline,
    upsample: ComputePipeline,
    resources: BloomResources,
    texture: u32,
    storage_images: Vec<u32>,
}

impl BloomPass {
    pub fn new(gpu: &mut GpuContext, binaries: &ShaderBinaries, extent: vk::Extent2D) -> anyhow::Result<BloomPass> {
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
        let resources = BloomResources::new(gpu, extent, BLOOM_LEVELS)?;
        let texture = gpu
            .bindless
            .add_texture(&gpu.device, resources.image.view(), vk::ImageLayout::GENERAL)?;
        let storage_images = resources
            .level_views
            .iter()
            .map(|view| gpu.bindless.add_storage_image(&gpu.device, *view))
            .collect::<anyhow::Result<_>>()?;

        Ok(BloomPass {
            downsample,
            upsample,
            resources,
            texture,
            storage_images,
        })
    }

    pub fn texture(&self) -> u32 {
        self.texture
    }

    fn dispatch(&self, frame: &FrameContext, pipeline: &ComputePipeline, step: &BloomStep) {
        let recorder = &frame.recorder;
        recorder.bind_compute(pipeline, frame.descriptor_set);
        recorder.push_compute_constants(
            pipeline,
            &BloomPushConstants {
                source_texture: step.source_texture,
                source_level: step.source_level,
                target_image: self.storage_images[step.target_level],
                karis_average: u32::from(step.karis_average),
            },
        );
        recorder.dispatch(self.resources.level_extents[step.target_level], WORKGROUP_SIZE);
        recorder.bloom_between_passes(self.resources.image.handle());
    }
}

impl RenderPass for BloomPass {
    fn record(&mut self, frame: &FrameContext) -> anyhow::Result<()> {
        if !frame.render.bloom.enabled {
            return Ok(());
        }

        frame.recorder.bloom_start(self.resources.image.handle());

        for level in 0..BLOOM_LEVELS as usize {
            let first = level == 0;
            let step = BloomStep {
                source_texture: if first { frame.scene_color_texture } else { self.texture },
                source_level: if first { 0 } else { level as u32 - 1 },
                target_level: level,
                karis_average: first,
            };
            self.dispatch(frame, &self.downsample, &step);
        }

        for level in (0..BLOOM_LEVELS as usize - 1).rev() {
            let step = BloomStep {
                source_texture: self.texture,
                source_level: level as u32 + 1,
                target_level: level,
                karis_average: false,
            };
            self.dispatch(frame, &self.upsample, &step);
        }

        frame.recorder.bloom_to_fragment(self.resources.image.handle());

        Ok(())
    }

    fn resize(&mut self, gpu: &mut GpuContext, targets: &RenderTargets) -> anyhow::Result<()> {
        let resources = BloomResources::new(gpu, targets.extent, BLOOM_LEVELS)?;
        gpu.bindless.set_texture(
            &gpu.device,
            self.texture,
            resources.image.view(),
            vk::ImageLayout::GENERAL,
        );

        for (slot, view) in self.storage_images.iter().zip(&resources.level_views) {
            gpu.bindless.set_storage_image(&gpu.device, *slot, *view);
        }

        let mut old = std::mem::replace(&mut self.resources, resources);
        unsafe { old.destroy(gpu) };

        Ok(())
    }

    unsafe fn destroy(&mut self, gpu: &mut GpuContext) {
        unsafe {
            self.downsample.destroy(&gpu.device);
            self.upsample.destroy(&gpu.device);
            self.resources.destroy(gpu);
        }
    }
}
